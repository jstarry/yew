#[wasm_bindgen]
fn main() {
    web_logger::init();
    log::info!("Initializing yew...");
    yew::start_app::<nested_list::Model>();
}
