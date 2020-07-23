# Changelog
 
## Unreleased
 
  - …

## annotate-snippets 0.8.0 (April 14, 2020)

  - Replace `ansi_term` with `yansi-term` for improved performance. (#30)
  - Turn `Snippet` and `Slice` to work on borrowed slices, rather than Strings. (#32)
  - Fix `\r\n` end of lines. (#29)

## annotate-snippets 0.7.0 (March 30, 2020)

  - Refactor API to use `fmt::Display` (#27)
  - Fix SourceAnnotation range (#27)
  - Fix column numbers (#22)
  - Derive `PartialEq` for `AnnotationType` (#19)
  - Update `ansi_term` to 0.12.

## annotate-snippets 0.6.1 (July 23, 2019)

  - Fix too many anonymized line numbers (#5)
 
## annotate-snippets 0.6.0 (June 26, 2019)
 
  - Add an option to anonymize line numbers (#3)
  - Transition the crate to rust-lang org.
  - Update the syntax to Rust 2018 idioms. (#4)
