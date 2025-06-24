use std::ops::Deref;

use data::*;
use yew::prelude::*;
use print::PrintTable;
use business_tab::BusinessTab;

use crate::{events::BusinessEvents, persistence::{read_settings, write_settings}, scheduler::{Controls, ScheduleCopy, Table}, settings::Settings};

mod automation;
mod business_tab;
mod data;
mod events;
mod persistence;
mod print;
mod settings;
mod scheduler;

pub type BusinessContext = UseReducerHandle<Business>;
pub type SettingsContext = UseStateHandle<Settings>;
pub type TabContext = UseStateHandle<Tabs>;
pub type Sort = UseStateHandle<EmployeeSort>;

#[derive(PartialEq)]
pub enum Tabs {
    Schedule,
    Business,
    Settings,
} impl Tabs {
    pub fn curr_tab(&self, tab: Tabs) -> Option<AttrValue> {
        if tab.eq(self) {
            Some("mui--is-active".into())
        } else {
            None
        }
    }
}

#[function_component]
fn App() -> Html {
    let settings = use_state_eq(|| Settings::default());
    let business = use_reducer_eq(|| Business::sample(settings.deref()));
    
    {
        let (business, settings) = (business.clone(), settings.clone());
        use_effect_with((), move |_| {
            let read = read_settings();
            match read.1 {
                Some(s) => settings.set(s),
                None => (),
            }
            match read.0 {
                Some(mut b) => {
                    b.init(settings.app.open.clone(), settings.app.close.clone(), settings.app.block_size.clone());
                    business.dispatch(BusinessEvents::InitFromHash { new_business: b });
                },
                None => (),
            }
        });
    }
    
    let tab = use_state_eq(|| Tabs::Schedule);
    let sort_table = use_state_eq(|| EmployeeSort::Name);
    let sort_settings = use_state_eq(|| EmployeeSort::Name);

    write_settings(settings.deref());
    // let mut tab_styles = vec![None; 3];
    // match tab.deref() {
    //     Tabs::Schedule => tab_styles[0] = Some("mui--is-active"),
    //     Tabs::Business => tab_styles[1] = Some("mui--is-active"),
    //     Tabs::Settings => tab_styles[2] = Some("mui--is-active"),
    // }

    html! {<ContextProvider<BusinessContext> context={business}> 
        <ContextProvider<SettingsContext> context={settings}>
        <PrintTable />
        <ContextProvider<TabContext> context={tab.clone()}>
            <TabBar />
        </ContextProvider<TabContext>>
        <ContextProvider<Sort> context={sort_table}>
            <div class={classes!("mui-tabs__pane", tab.curr_tab(Tabs::Schedule))}>
                <div class={"pane-content"}>
                    <Table />
                    <br />
                    <Controls />
                    <br />
                    <ScheduleCopy />
                </div>
            </div>
        </ContextProvider<Sort>>
        <ContextProvider<Sort> context={sort_settings}>
            <div class={classes!("mui-tabs__pane", tab.curr_tab(Tabs::Business))}>
                <div class={"pane-content"}>
                    <BusinessTab />
                </div>
            </div>
        </ContextProvider<Sort>>
        <div class={classes!("mui-tabs__pane", tab.curr_tab(Tabs::Settings))}>
            <div class={"pane-content settings-tab"}>
                <settings::SettingsTab />
            </div>
        </div>
        </ContextProvider<SettingsContext>>
    </ContextProvider<BusinessContext>>}
}

#[function_component]
fn TabBar() -> Html {
    let tab = use_context::<TabContext>().expect("Tab context not found");
    let business_context = tab.clone();
    let schedule_context = tab.clone();
    let settings_context = tab.clone();
    // html!(<div><table><tr>
    //     <td onclick={move |_| {settings_context.set(Tabs::Settings);}}>
    //         {"Settings"}
    //     </td>
    //     <td onclick={move |_| {main_context.set(Tabs::Main);}}>
    //         {"Main"}
    //     </td>
    // </tr></table></div>)
    // let mut tab_styles = vec![None, None];
    // match tab.deref() {
    //     Tabs::Main => tab_styles[0] = Some("mui--is-active"),
    //     Tabs::Settings => tab_styles[1] = Some("mui--is-active"),
    // };
    html!(
        <ul class="mui-tabs__bar mui-tabs__bar--justified">
            <li class={tab.curr_tab(Tabs::Business)} onclick={move |_| {business_context.set(Tabs::Business);}}>{"Business"}</li>
            <li class={tab.curr_tab(Tabs::Schedule)} onclick={move |_| {schedule_context.set(Tabs::Schedule);}}>{"Schedule"}</li>
            <li class={tab.curr_tab(Tabs::Settings)} onclick={move |_| {settings_context.set(Tabs::Settings);}}>{"Settings"}</li>
        </ul>
    )
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    println!("Hello, world!");
    yew::Renderer::<App>::new().render();
}
