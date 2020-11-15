use instant::Instant;
use std::time::Duration;
use yew::{
    prelude::*,
    services::interval::{IntervalService, IntervalTask},
};

const RESOLUTION: u64 = 500;
const MIN_INTERVAL_MS: u64 = 50;

pub enum Msg {
    Tick,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub duration_ms: u64,
    pub on_complete: Callback<()>,
    #[prop_or_default]
    pub on_progress: Callback<f64>,
}

pub struct ProgressDelay {
    _task: IntervalTask,
    start: Instant,
    value: f64,
}

impl Component for ProgressDelay {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let interval = (ctx.props.duration_ms / RESOLUTION).min(MIN_INTERVAL_MS);
        let task =
            IntervalService::spawn(Duration::from_millis(interval), ctx.callback(|_| Msg::Tick));
        Self {
            _task: task,
            start: Instant::now(),
            value: 0.0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                let duration = ctx.props.duration_ms;
                let elapsed = self.start.elapsed().as_millis() as u64;
                self.value = elapsed as f64 / duration as f64;

                if elapsed > duration {
                    ctx.props.on_complete.emit(());
                    self.start = Instant::now();
                } else {
                    ctx.props.on_progress.emit(self.value);
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let value = self.value;
        html! {
            <progress class="progress is-primary" value=value max=1.0>
                { format!("{:.0}%", 100.0 * value) }
            </progress>
        }
    }
}
