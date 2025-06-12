use std::str::FromStr;

use log::error;
use yew::prelude::*;

use crate::{events::BusinessEvents, BusinessContext};

pub type HeldBlock = UseStateHandle<TimeBlock>;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct TimeBlock {
    pub emp_id: usize,
    pub time_index: usize,
    pub role: usize,
    pub len: usize,
    pub len_index: usize
} impl TimeBlock {
    pub fn new_simple(emp_id: usize, time_index: usize, role: usize) -> TimeBlock {
        TimeBlock { emp_id, time_index, role, len: 1, len_index: 0 }
    }
}impl FromStr for TimeBlock {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let set: Vec<&str> = s.split(",").collect();
        
        Ok(TimeBlock { 
            emp_id: set[0].parse()?, 
            time_index: set[1].parse()?,
            role: set[2].parse()?,
            len: set[3].parse()?,
            len_index: set[4].parse()?,
        })
    }
} impl ToString for TimeBlock {
    fn to_string(&self) -> String {
        format!("{},{},{},{},{}", self.emp_id, self.time_index, self.role, self.len, self.len_index)
    }
}

pub fn static_block(block: TimeBlock, style: Option<String>, business: BusinessContext, held_block: HeldBlock) -> Html {

    let ondrop = drop_handler(block, business, held_block.clone());

    html!(
        <div class="time-block" style={style} ondragover={drag_over_handler} ondrop={ondrop}>
            {"s"}
        </div>
    )
}

pub fn drag_block(block: TimeBlock, style: Option<String>, business: BusinessContext, held_block: HeldBlock) -> Html {

    let drag_start_handler = drag_start_wrapper(block.clone());
    let ondrop = drop_handler(block, business, held_block.clone());

    html!(
        <div class="time-block" style={style} draggable="true" ondragstart={drag_start_handler} ondragover={drag_over_handler} ondrop={ondrop}>
            {"d"}
        </div>
    )
}

pub fn multi_block(mut block: TimeBlock, style: Option<String>, business: BusinessContext, held_block: HeldBlock) -> Html {

    let ondragstart = drag_start_wrapper(block.clone());

    let mut single_blocks: Vec<Html> = vec![];
    let blocks = block.len.clone();
    for _ in 0..blocks {
        let onclick;
        {
            let block = block.clone();
            let held_block = held_block.clone();
            onclick = move |_| held_block.set(block.clone());
        }
        single_blocks.push(html!(
            <div class="time-block multi-block"  style={style.clone()} ondragover={drag_over_handler} ondrop={drop_handler(block.clone(), business.clone(), held_block.clone())} onmousedown={onclick}>
                {"m"}
            </div>
        ));
        block.time_index += 1;
        block.len_index += 1;
    }

    html!(
        <div draggable="true" ondragstart={ondragstart}>
            {single_blocks}
        </div>
    )
}

fn drag_start_wrapper(block: TimeBlock) -> impl Fn(DragEvent) {
    move |ev| {
        match ev.data_transfer().unwrap().set_data("TimeBlock", &block.to_string()) {
            Ok(_) => (),
            Err(_) => error!("Failed to set drag data transfer!"),
        }
    }
}

fn drag_over_handler(ev: DragEvent) {
    ev.prevent_default();
}

fn drop_handler(target_block: TimeBlock, business: BusinessContext, held_block: HeldBlock) -> impl Fn(DragEvent) {
    move |ev: DragEvent| {
        let drag_block;
        match ev.data_transfer().unwrap().get_data("TimeBlock") {
            Ok(x) => {
                match TimeBlock::from_str(&x) {
                    Ok(x) => drag_block = x,
                    Err(e) => {error!("Failed to covert drag data transfer to time block {:#?}", e); return;},
                }
            },
            Err(e) => {error!("Failed to get drag data transfer {:#?}", e); return;},
        }
        business.dispatch(BusinessEvents::DragAssignBlock { target_block: target_block.clone(), drag_block, held_block: held_block.clone() });
    }
}