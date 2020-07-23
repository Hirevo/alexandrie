use async_std::io::Cursor;
use async_test::TestCase;
use http_types::{Body, Response, StatusCode};

mod common;

const REQUEST: &'static str = concat![
    "GET / HTTP/1.1\r\n",
    "host: example.com\r\n",
    "user-agent: curl/7.54.0\r\n",
    "content-type: text/plain\r\n",
    "\r\n",
];

const TEXT: &'static str = concat![
    "Eveniet delectus voluptatem in placeat modi. Qui nulla sunt aut non voluptas temporibus accusamus rem. Qui soluta nisi qui accusantium excepturi voluptatem. Ab rerum maiores neque ut expedita rem.",
    "Et neque praesentium eligendi quaerat consequatur asperiores dolorem. Pariatur tempore quidem animi consequuntur voluptatem quos. Porro quo ipsa quae suscipit. Doloribus est qui facilis ratione. Delectus ex perspiciatis ab alias et quisquam non est.",
    "Id dolorum distinctio distinctio quos est facilis commodi velit. Ex repudiandae aliquam eos voluptatum et. Provident qui molestiae molestiae nostrum voluptatum aperiam ut. Quis repellendus quidem mollitia aut recusandae laboriosam.",
    "Corrupti cupiditate maxime voluptatibus totam neque facilis. Iure deleniti id incidunt in sunt suscipit ea. Hic ullam qui doloribus tempora voluptas. Unde id debitis architecto beatae dolores autem et omnis. Impedit accusamus laudantium voluptatem ducimus.",
    "Eos maxime hic aliquid accusantium. Et voluptas sit accusamus modi natus. Et voluptatem sequi ea et provident voluptatum minus voluptas. Culpa aliquam architecto consequatur animi.",
];

const RESPONSE: &'static str = concat![
    "HTTP/1.1 200 OK\r\n",
    "transfer-encoding: chunked\r\n",
    "date: {DATE}\r\n",
    "content-type: application/octet-stream\r\n",
    "\r\n",
    "458\r\n",
    "Eveniet delectus voluptatem in placeat modi. Qui nulla sunt aut non voluptas temporibus accusamus rem. Qui soluta nisi qui accusantium excepturi voluptatem. Ab rerum maiores neque ut expedita rem.",
    "Et neque praesentium eligendi quaerat consequatur asperiores dolorem. Pariatur tempore quidem animi consequuntur voluptatem quos. Porro quo ipsa quae suscipit. Doloribus est qui facilis ratione. Delectus ex perspiciatis ab alias et quisquam non est.",
    "Id dolorum distinctio distinctio quos est facilis commodi velit. Ex repudiandae aliquam eos voluptatum et. Provident qui molestiae molestiae nostrum voluptatum aperiam ut. Quis repellendus quidem mollitia aut recusandae laboriosam.",
    "Corrupti cupiditate maxime voluptatibus totam neque facilis. Iure deleniti id incidunt in sunt suscipit ea. Hic ullam qui doloribus tempora voluptas. Unde id debitis architecto beatae dolores autem et omnis. Impedit accusamus laudantium voluptatem ducimus.",
    "Eos maxime hic aliquid accusantium. Et voluptas sit accusamus modi natus. Et voluptatem sequi ea et provident voluptatum minus voluptas. Culpa aliquam architecto consequatur animi.",
    "\r\n",
    "0",
    "\r\n",
    "\r\n",
];

#[async_std::test]
async fn server_chunked_large() {
    let case = TestCase::new(REQUEST, "").await;
    async_h1::accept(case.clone(), |_| async {
        let mut res = Response::new(StatusCode::Ok);
        let body = Cursor::new(TEXT.to_owned());
        res.set_body(Body::from_reader(body, None));
        Ok(res)
    })
    .await
    .unwrap();
    case.assert_writer_with(RESPONSE, common::munge_date).await;
}
