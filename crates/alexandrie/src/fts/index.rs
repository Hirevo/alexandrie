use std::convert::TryFrom;
use std::num::NonZeroUsize;
use std::sync::RwLock;

use diesel::prelude::*;
use log::{debug, info, warn};
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, QueryParser};
use tantivy::schema::{NumericOptions, Schema, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::{
    Language, LowerCaser, RawTokenizer, SimpleTokenizer, StopWordFilter, TextAnalyzer,
    TokenizerManager,
};
use tantivy::{
    Index as TantivyIndex, IndexReader, IndexWriter, Opstamp, ReloadPolicy, TantivyError, Term,
};
use tantivy_analysis_contrib::commons::EdgeNgramTokenFilter;

use crate::config::SearchConfig;
use crate::db::models::Crate;
use crate::db::schema::*;
use crate::db::Database;
use crate::error::Error;
use crate::fts::TantivyDocument;

const NUMBER_RESULT_PER_PAGE: i64 = 1000;

type CrateKeywordCategory = (Vec<Crate>, Vec<(i64, String)>, Vec<(i64, String)>);

/// Helper for using Tantivy
pub struct Tantivy {
    index_reader: IndexReader,
    /// There can only be one index writer at a time (see https://tantivy-search.github.io/examples/basic_search.html)
    /// so we keep only one here. It has its own pool.
    index_writer: RwLock<IndexWriter>,
    /// Index schema
    schema: Schema,
    /// Search tokenizer manager
    search_tokenizer_manager: TokenizerManager,
}

impl TryFrom<SearchConfig> for Tantivy {
    type Error = crate::error::Error;

    fn try_from(search: SearchConfig) -> Result<Self, Self::Error> {
        let analyzer_name = "alexandrie";
        // Not tokenized
        let analyzer_name_full = "alexandrie_full";
        // Prefix
        let analyzer_prefix_name = "alexandrie_prefix";

        // Create index directory
        let path = search.path.as_str();
        std::fs::create_dir_all(path)?;
        let directory = MmapDirectory::open(path)?;

        // Index options for all fields (perhaps keywords and category could have another analysis & options)
        let options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(analyzer_name)
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let options_full = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(analyzer_name_full)
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let options_prefixes = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(analyzer_prefix_name)
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let id_options = NumericOptions::default().set_stored().set_indexed();

        // Schema of a document, we index and store (though storing isn't really necessary):
        // * name : crate's name
        // * description: crate's description
        // * categories: crate's categories
        // * keywords: crate's keywords
        let mut schema_builder = Schema::builder();
        // Easier with i64 than u64 because IDs are i64 i db module.
        schema_builder.add_i64_field(super::ID_FIELD_NAME, id_options);

        // For these fields, we could avoid storing original data if we get things in the database right
        // after searching indices
        schema_builder.add_text_field(super::NAME_FIELD_NAME, options.clone());
        schema_builder.add_text_field(super::NAME_FIELD_NAME_FULL, options_full.clone());
        schema_builder.add_text_field(super::NAME_FIELD_PREFIX_NAME, options_prefixes);
        schema_builder.add_text_field(super::DESCRIPTION_FIELD_NAME, options.clone());
        schema_builder.add_text_field(super::CATEGORY_FIELD_NAME, options_full);
        schema_builder.add_text_field(super::KEYWORD_FIELD_NAME, options);
        let schema = schema_builder.build();

        // Analysis a tokenizer that tokenizes on non-alphanumeric characters
        // A filter that removes common english words (the, a, ...etc)
        // A filter that lowercase words
        let stop_words =
            StopWordFilter::new(Language::English).ok_or(Self::Error::EmptyStopWord)?;
        let analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(stop_words)
            .filter(LowerCaser)
            .build();

        let analyzer_full = TextAnalyzer::builder(RawTokenizer::default())
            .filter(LowerCaser)
            .build();

        let analyzer_prefix = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .filter(EdgeNgramTokenFilter::new(NonZeroUsize::new(1).unwrap(), None, false).unwrap())
            .build();

        let index = TantivyIndex::open_or_create(directory, schema.clone())?;
        // Register analyzer
        index.tokenizers().register(analyzer_name, analyzer.clone());
        index
            .tokenizers()
            .register(analyzer_name_full, analyzer_full.clone());
        index
            .tokenizers()
            .register(analyzer_prefix_name, analyzer_prefix);

        // Create an analyzer manager for search: on name prefix we do not want to apply
        // the edge ngram filter: more efficient & less noise
        // We need to have the other tokenizer registered so that search work on any field properly
        let search_tokenizer_manager = TokenizerManager::new();
        search_tokenizer_manager.register(analyzer_name, analyzer);
        search_tokenizer_manager.register(analyzer_name_full, analyzer_full);
        search_tokenizer_manager.register(
            analyzer_prefix_name,
            TextAnalyzer::builder(SimpleTokenizer::default())
                .filter(LowerCaser)
                .build(),
        );

        // Get an index writer with 50MB of heap
        let index_writer = RwLock::new(index.writer(50_000_000)?);

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        Ok(Self {
            index_reader,
            index_writer,
            schema,
            search_tokenizer_manager,
        })
    }
}

impl Tantivy {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Method that create or update a document in Tantivy index. As there is no update, we need
    /// to first delete the document then create a new document.
    pub fn create_or_update(&self, document: TantivyDocument) -> Result<(), Error> {
        let id = document.id();
        let document = document.try_into(&self.schema)?;
        let field = self.schema.get_field(super::ID_FIELD_NAME)?;
        let term = Term::from_field_i64(field, id);
        let index_writer = self
            .index_writer
            .read()
            .map_err(|error| Error::PoisonedError(error.to_string()))?;
        index_writer.delete_term(term);
        index_writer.add_document(document)?;

        Ok(())
    }

    pub fn delete_all_documents(&self) -> Result<Opstamp, Error> {
        let index_writer = self
            .index_writer
            .read()
            .map_err(|error| Error::PoisonedError(error.to_string()))?;
        Ok(index_writer.delete_all_documents()?)
    }

    /// Commit all pending changes inside the index.
    pub fn commit(&self) -> Result<Opstamp, Error> {
        let mut index_writer = self
            .index_writer
            .write()
            .map_err(|error| Error::PoisonedError(error.to_string()))?;
        Ok(index_writer.commit()?)
    }

    /// Search document by default through all crate's name index. This allows having search
    /// as you type (using prefixes) while increasing relevance when there's a matching word
    /// or if the whole text matches a crate's name (using the other crate's name indices).
    pub fn suggest(&self, query: String, limit: usize) -> Result<Vec<String>, TantivyError> {
        let searcher = self.index_reader.searcher();

        let name = self.schema.get_field(super::NAME_FIELD_NAME).unwrap();
        let name_full = self.schema.get_field(super::NAME_FIELD_NAME_FULL).unwrap();
        let name_prefix = self
            .schema
            .get_field(super::NAME_FIELD_PREFIX_NAME)
            .unwrap();

        let mut query_parser = QueryParser::new(
            self.schema.clone(),
            vec![name_full, name_prefix],
            self.search_tokenizer_manager.clone(),
        );

        query_parser.set_field_boost(name_full, 10.0);
        query_parser.set_field_boost(name, 5.0);
        query_parser.set_field_boost(name_prefix, 1.0);

        let query = query_parser.parse_query(&query)?;

        info!("Query : {:?}", query);

        let results = searcher.search(&query, &TopDocs::with_limit(limit))?;

        info!("Result : {:?}", results);

        let results = results
            .into_iter()
            .map(|(score, doc_address)| {
                let retrieve_doc = searcher.doc(doc_address).unwrap();

                let x = retrieve_doc.get_all(name).next();
                if let Some(n) = x {
                    info!("Score : {} / Crate : {:?}", score, n);
                }
                x.cloned()
            })
            .filter_map(|v| v.and_then(|i| i.as_text().map(|t| t.to_owned())))
            .collect();

        Ok(results)
    }

    /// Search documents. Return document count & database IDs.
    pub fn search<Q: AsRef<str>>(
        &self,
        query: Q,
        offset: usize,
        limit: usize,
    ) -> Result<(usize, Vec<i64>), TantivyError> {
        let query = query.as_ref().trim();

        let searcher = self.index_reader.searcher();

        let id = self.schema.get_field(super::ID_FIELD_NAME).unwrap();
        let name = self.schema.get_field(super::NAME_FIELD_NAME).unwrap();
        let name_full = self.schema.get_field(super::NAME_FIELD_NAME_FULL).unwrap();
        let description = self
            .schema
            .get_field(super::DESCRIPTION_FIELD_NAME)
            .unwrap();
        let categories = self.schema.get_field(super::CATEGORY_FIELD_NAME).unwrap();
        let keywords = self.schema.get_field(super::KEYWORD_FIELD_NAME).unwrap();

        let query = if query.is_empty() {
            Box::new(AllQuery)
        } else {
            let mut query_parser = QueryParser::for_index(
                searcher.index(),
                vec![name, name_full, description, categories, keywords],
            );

            // Exact matches (on name_full) have a big boost
            query_parser.set_field_boost(name_full, 10.0);
            query_parser.set_field_boost(name, 5.0);
            // Categories shouldn't be free (there is a list) so a nice boost
            query_parser.set_field_boost(categories, 1.0);
            // Keywords are free
            query_parser.set_field_boost(keywords, 0.5);
            // description & readme are full text they got a lower boost (if there is a match, that might not be relevant)
            query_parser.set_field_boost(description, 0.2);

            query_parser.parse_query(query)?
        };

        info!("Query offset={} query limit={}", offset, limit);

        let (count, results) = searcher.search(
            &query,
            &(Count, TopDocs::with_limit(limit).and_offset(offset)),
        )?;

        let results = results
            .into_iter()
            .filter_map(|(score, doc_address)| {
                let retrieve_doc = match searcher.doc(doc_address) {
                    Ok(retrieve_doc) => retrieve_doc,
                    Err(error) => {
                        warn!("Could not find document {doc_address:?} : {error}");
                        return None;
                    }
                };

                if let Some(name) = retrieve_doc.get_all(name).next() {
                    info!("Score : {} / Crate : {:?}", score, name);
                }

                let mut field = retrieve_doc.get_all(id);
                if let Some(x) = field.next() {
                    x.as_i64()
                } else {
                    warn!("Could not find field id");
                    None
                }
            })
            .collect();

        Ok((count, results))
    }

    pub async fn index_all(&self, repo: &Database) -> Result<(), Error> {
        info!("Index all crates");
        self.delete_all_documents()?;
        self.commit()?;
        let mut start: i64 = 0;
        let mut count_crate = 0;

        'indexing: loop {
            let result: Result<Option<CrateKeywordCategory>, Error> = repo
                .run(move |conn| {
                    debug!(
                        "Querying crates from {start} to {}",
                        start + NUMBER_RESULT_PER_PAGE
                    );
                    let krates = crates::table
                        .order_by(crates::id.asc())
                        .limit(NUMBER_RESULT_PER_PAGE)
                        .offset(start)
                        .load::<Crate>(conn)?;

                    debug!("{} crates fetched", krates.len());

                    if krates.is_empty() {
                        return Ok(None);
                    }

                    let ids = krates
                        .clone()
                        .into_iter()
                        .map(|c| c.id)
                        .collect::<Vec<i64>>();

                    debug!("Crates {:?}", ids);

                    let keywords = keywords::table
                        .inner_join(crate_keywords::table)
                        .select((crate_keywords::crate_id, keywords::name))
                        .filter(crate_keywords::crate_id.eq_any(&ids))
                        .order_by(crate_keywords::crate_id.asc())
                        .load::<(i64, String)>(conn)?;

                    let categories = categories::table
                        .inner_join(crate_categories::table)
                        .select((crate_categories::crate_id, categories::name))
                        .filter(crate_categories::crate_id.eq_any(&ids))
                        .order_by(crate_categories::crate_id.asc())
                        .load::<(i64, String)>(conn)?;

                    Ok(Some((krates, keywords, categories)))
                })
                .await;

            let result = result?;

            if let Some((krates, keywords, categories)) = result {
                start += krates.len() as i64;
                let mut keywords_iterator = keywords.into_iter().peekable();
                let mut categories_iterator = categories.into_iter().peekable();

                let mut current_keyword: Option<(i64, String)> = keywords_iterator.next();
                let mut current_category: Option<(i64, String)> = categories_iterator.next();

                for krate in krates.into_iter() {
                    debug!("crate {:?}", krate);
                    // Create a document with database ID and crate name
                    let id = krate.id;
                    let name = krate.name.clone();

                    let mut doc: TantivyDocument = krate.into();

                    // Skip keywords that might be orphan and add keywords that match ids
                    while let Some((crate_id, keyword)) = current_keyword {
                        if crate_id > id {
                            current_keyword = Some((crate_id, keyword));
                            break;
                        }

                        if crate_id == id {
                            doc.add_keyword(keyword);
                        }

                        current_keyword = keywords_iterator.next();
                    }

                    // Skip categories that might be orphan and add categories that match ids
                    while let Some((crate_id, category)) = current_category {
                        if crate_id > id {
                            current_category = Some((crate_id, category));
                            break;
                        }

                        if crate_id == id {
                            doc.add_category(category);
                        }

                        current_category = categories_iterator.next();
                    }

                    // TODO get README

                    if let Err(error) = self.create_or_update(doc) {
                        warn!(
                            "Can't convert crate '{id}' ({name}) into Tantivy document : {error}"
                        );
                    }
                    count_crate += 1;

                    if count_crate % 1000 == 0 {
                        info!("{} crates indexed", count_crate);
                    }
                }
            } else {
                info!("End indexing {start} crates");
                self.commit()?;
                break 'indexing;
            }
        }

        Ok(())
    }
}
