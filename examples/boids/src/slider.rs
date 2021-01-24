#![allow(deprecated)]

use std::cell::Cell;
use yew::component::{Component, Context};
use yew::{html, Callback, Html, InputData, Properties};

thread_local! {
    static SLIDER_ID: Cell<usize> = Cell::default();
}
fn next_slider_id() -> usize {
    SLIDER_ID.with(|cell| cell.replace(cell.get() + 1))
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub label: &'static str,
    pub value: f64,
    pub onchange: Callback<f64>,
    #[prop_or_default]
    pub precision: Option<usize>,
    #[prop_or_default]
    pub percentage: bool,
    #[prop_or_default]
    pub min: f64,
    pub max: f64,
    #[prop_or_default]
    pub step: Option<f64>,
}

pub struct Slider {
    id: usize,
}

impl Component for Slider {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: next_slider_id(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Props {
            label,
            value,
            ref onchange,
            precision,
            percentage,
            min,
            max,
            step,
        } = *ctx.props;

        let precision = precision.unwrap_or_else(|| if percentage { 1 } else { 0 });

        let display_value = if percentage {
            format!("{:.p$}%", 100.0 * value, p = precision)
        } else {
            format!("{:.p$}", value, p = precision)
        };

        let id = format!("slider-{}", self.id);
        let step = step.unwrap_or_else(|| {
            let p = if percentage { precision + 2 } else { precision };
            10f64.powi(-(p as i32))
        });

        html! {
            <div class="slider">
                <label for=id class="slider__label">{ label }</label>
                <input type="range"
                    id=id
                    class="slider__input"
                    min=min max=max step=step
                    oninput=onchange.reform(|data: InputData| data.value.parse().unwrap())
                    value=value
                />
                <span class="slider__value">{ display_value }</span>
            </div>
        }
    }
}
