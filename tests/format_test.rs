extern crate serde_derive;
extern crate yew;

mod wasm_helper;

use serde_derive::{Deserialize, Serialize};
use yew::format::{Binary, Json, Text};

#[test]
#[wasm_helper::test]
fn json_format() {
    #[derive(Serialize, Deserialize)]
    struct Data {
        value: u8,
    }

    let Json(data): Json<Result<Data, _>> = Json::from(Ok(r#"{"value": 123}"#.to_string()));
    let data = data.unwrap();
    assert_eq!(data.value, 123);

    let _stored: Text = Json(&data).into();
    let _stored: Binary = Json(&data).into();
}
