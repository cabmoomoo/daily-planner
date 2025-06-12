#![allow(dead_code)]

use std::ops::Deref;

use data::*;
use log::info;
use yew::prelude::*;
use settings::Settings;

use crate::scheduler::{Controls, Table};

mod data;
mod events;
mod settings;
mod scheduler;

pub type BusinessContext = UseReducerHandle<Business>;
pub type TabContext = UseStateHandle<Tabs>;

#[derive(PartialEq)]
pub enum Tabs {
    Main,
    Settings
}

#[function_component]
fn App() -> Html {
    let business = use_reducer(|| Business::sample());
    let tab = use_state_eq(|| Tabs::Main);

    // info!("{:#?}", business);

    let mut tab_styles = vec![None, None];
    match tab.deref() {
        Tabs::Main => tab_styles[0] = Some("mui--is-active"),
        Tabs::Settings => tab_styles[1] = Some("mui--is-active"),
    }

    html! {<ContextProvider<BusinessContext> context={business}>
        <ContextProvider<TabContext> context={tab.clone()}>
            <TabBar />
        </ContextProvider<TabContext>>
        <div class={classes!("mui-tabs__pane", tab_styles[0])}>
            <div class={"pane-content"}>
                <Table />
                <br />
                <Controls />
                <br />

            </div>
        </div>
        <div class={classes!("mui-tabs__pane", tab_styles[1])}>
            <Settings />
        </div>
    </ContextProvider<BusinessContext>>}
}

#[function_component]
fn TabBar() -> Html {
    let tab = use_context::<TabContext>().expect("Tab context not found");
    let settings_context = tab.clone();
    let main_context = tab.clone();
    // html!(<div><table><tr>
    //     <td onclick={move |_| {settings_context.set(Tabs::Settings);}}>
    //         {"Settings"}
    //     </td>
    //     <td onclick={move |_| {main_context.set(Tabs::Main);}}>
    //         {"Main"}
    //     </td>
    // </tr></table></div>)
    let mut tab_styles = vec![None, None];
    match tab.deref() {
        Tabs::Main => tab_styles[0] = Some("mui--is-active"),
        Tabs::Settings => tab_styles[1] = Some("mui--is-active"),
    };
    html!(
        <ul class="mui-tabs__bar mui-tabs__bar--justified">
            <li class={tab_styles[1]} onclick={move |_| {settings_context.set(Tabs::Settings);}}>{"Settings"}</li>
            <li class={tab_styles[0]} onclick={move |_| {main_context.set(Tabs::Main);}}>{"Main"}</li>
        </ul>
    )
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    println!("Hello, world!");
    yew::Renderer::<App>::new().render();
}
