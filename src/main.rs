#![allow(dead_code)]

use controls::*;
use data::*;
use yew::prelude::*;
use table::Table;

mod controls;
mod data;
mod events;
mod table;

pub type BusinessContext = UseReducerHandle<Business>;

#[function_component]
fn App() -> Html {
    let business = use_reducer(|| Business::sample());
    // business.assign_block(3, 2, vec![3,4,5,6]);
    // business.assign_block(3, 3, vec![4,5,6,7]);
    // business.assign_block(2, 2, vec![0,1,2,3]);

    html! {
        <ContextProvider<BusinessContext> context={business}>
        <Table />
        <br />
        <Controls />
        </ContextProvider<BusinessContext>>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    println!("Hello, world!");
    yew::Renderer::<App>::new().render();
}
