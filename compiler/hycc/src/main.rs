use std::collections::HashMap;

use hycc_pipeline::pipeline;

fn main() {
    type Test<T> = HashMap<T, usize>;
    let x: Test<&str> = HashMap::new();

    pipeline::start("hyc-tests/collection/collection.hyc");
}
