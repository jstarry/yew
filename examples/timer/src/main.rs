use std::time::Duration;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::{ConsoleService, Task, TimeoutService};
use yew::{html, Callback, Component, Context, Html, ShouldRender};

pub enum Msg {
    StartTimeout,
    StartInterval,
    Cancel,
    Done,
    Tick,
    UpdateTime,
}

pub struct Model {
    job: Option<Box<dyn Task>>,
    time: String,
    messages: Vec<&'static str>,
    _standalone: (IntervalTask, IntervalTask),
}

impl Model {
    fn get_current_time() -> String {
        let date = js_sys::Date::new_0();
        String::from(date.to_locale_time_string("en-US"))
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let standalone_handle = IntervalService::spawn(
            Duration::from_secs(10),
            // This callback doesn't send any message to a scope
            Callback::from(|_| {
                ConsoleService::debug("Example of a standalone callback.");
            }),
        );

        let clock_handle =
            IntervalService::spawn(Duration::from_secs(1), ctx.callback(|_| Msg::UpdateTime));

        Self {
            job: None,
            time: Model::get_current_time(),
            messages: Vec::new(),
            _standalone: (standalone_handle, clock_handle),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartTimeout => {
                let handle =
                    TimeoutService::spawn(Duration::from_secs(3), ctx.callback(|_| Msg::Done));
                self.job = Some(Box::new(handle));

                self.messages.clear();
                ConsoleService::clear();

                self.messages.push("Timer started!");
                ConsoleService::time_named("Timer");
                true
            }
            Msg::StartInterval => {
                let handle =
                    IntervalService::spawn(Duration::from_secs(1), ctx.callback(|_| Msg::Tick));
                self.job = Some(Box::new(handle));

                self.messages.clear();
                ConsoleService::clear();

                self.messages.push("Interval started!");
                true
            }
            Msg::Cancel => {
                self.job = None;
                self.messages.push("Canceled!");
                ConsoleService::warn("Canceled!");
                true
            }
            Msg::Done => {
                self.job = None;
                self.messages.push("Done!");

                ConsoleService::group();
                ConsoleService::info("Done!");
                ConsoleService::time_named_end("Timer");
                ConsoleService::group_end();
                true
            }
            Msg::Tick => {
                self.messages.push("Tick...");
                ConsoleService::count_named("Tick");
                true
            }
            Msg::UpdateTime => {
                self.time = Model::get_current_time();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let has_job = self.job.is_some();
        html! {
            <>
                <div id="buttons">
                    <button disabled=has_job onclick=ctx.callback(|_| Msg::StartTimeout)>
                        { "Start Timeout" }
                    </button>
                    <button disabled=has_job onclick=ctx.callback(|_| Msg::StartInterval)>
                        { "Start Interval" }
                    </button>
                    <button disabled=!has_job onclick=ctx.callback(|_| Msg::Cancel)>
                        { "Cancel!" }
                    </button>
                </div>
                <div id="wrapper">
                    <div id="time">
                        { &self.time }
                    </div>
                    <div id="messages">
                        { for self.messages.iter().map(|message| html! { <p>{ message }</p> }) }
                    </div>
                </div>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
