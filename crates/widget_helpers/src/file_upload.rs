/// this was borrowed from
/// https://github.com/yewstack/yew/tree/8172b9ceacdcd7d4609e8ba00f758507a8bbc85d/examples/file_upload
/// and modified.
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use std::collections::HashMap;
use web_sys::{Event, HtmlInputElement};
use weblog::console_error;
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html, Properties};

pub enum Msg {
    Loaded(FileBytes),
    Files(Vec<File>),
}

pub struct FileBytes {
    pub filename: String,
    pub bytes: Vec<u8>,
}

pub struct FileUploadApp {
    readers: HashMap<String, FileReader>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub content_name: String,
    pub on_file_upload: Callback<FileBytes>,
}

impl Component for FileUploadApp {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(fb) => {
                self.readers.remove(&fb.filename);
                ctx.props().on_file_upload.emit(fb);
                false
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let filename = file.name();
                    let task = {
                        let filename = filename.clone();
                        let link = ctx.link().clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| match res {
                            Err(err) => {
                                console_error!(format!("gloo file read_as_bytes error: {}", err));
                            }
                            Ok(bytes) => {
                                link.send_message(Msg::Loaded(FileBytes { filename, bytes }))
                            }
                        })
                    };
                    self.readers.insert(filename, task);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div>
                    <h3 class="subtitle">{format!("upload {}", ctx.props().content_name)}</h3>
                    <input type="file" multiple=true onchange={ctx.link().callback(move |e: Event| {
                            let mut result = Vec::new();
                            let input: HtmlInputElement = e.target_unchecked_into();

                            if let Some(files) = input.files() {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .map(|v| web_sys::File::from(v.unwrap()))
                                    .map(File::from);
                                result.extend(files);
                            }
                            Msg::Files(result)
                        })}
                    />
                </div>
            </div>
        }
    }
}
