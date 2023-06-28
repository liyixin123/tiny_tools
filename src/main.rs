// 目标：写一个程序，将平时的SVN地址格式转换一下

#![windows_subsystem = "windows"]

mod tests;

use druid::widget::{Checkbox, CrossAxisAlignment, Flex, FlexParams, Label, ListIter, Tabs};
use druid::{widget::{Button, TextBox}, AppLauncher, Data, Lens, Widget, WidgetExt, WindowDesc, Application};

use druid::im::Vector;
use tracing::info;
use std::fmt::Write;

const BASE_URL: &str = "http://172.17.102.22:18080/svn/softwarerepo";

#[derive(Clone, Data, Lens)]
struct TextBoxData {
    text: String,
}

impl ListIter<TextBoxData> for SVNAddress {
    fn for_each(&self, mut cb: impl FnMut(&TextBoxData, usize)) {
        self.new_addrs.iter().enumerate().for_each(|(index, item)| {
            cb(item, index);
        })
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut TextBoxData, usize)) {
        self.new_addrs.iter_mut().enumerate().for_each(|(index, item)| {
            cb(item, index);
        });
    }

    fn data_len(&self) -> usize {
        self.new_addrs.len()
    }
}

#[derive(Data, Clone, Lens)]
struct SVNAddress {
    old: String,
    new_addrs: Vector<TextBoxData>,
    name: String,
    read_write: bool,
}


impl SVNAddress {
    fn new() -> SVNAddress {
        SVNAddress {
            old: "".to_string(),
            new_addrs: Vector::new(),
            name: String::new(),
            read_write: true,
        }
    }
    fn update(&mut self) {
        self.new_addrs.clear();
        let srcs = extract_substrings_containing_base_url(self.old.as_str());
        if srcs.is_empty() || !srcs.first().unwrap().starts_with("http") {
            self.new_addrs.push_back(
                TextBoxData {
                    text: "格式错误".to_string()
                });
        }

        for x in srcs {
            self.new_addrs.push_back(TextBoxData { text: convert_address(&x) });
        }
    }
    fn merged_new_addr(&self) -> String {
        let mut builder = String::new();
        for textbox in &self.new_addrs {
            if !builder.is_empty() {
                builder.push('\n');
            }
            write!(builder, "{}", &textbox.text).expect("write failed");
        }
        builder
    }
}


fn extract_substrings_containing_base_url(input_str: &str) -> Vec<String> {
    input_str
        .split_whitespace()
        .filter(|s| s.contains(BASE_URL))
        .map(|s| s.to_string())
        .collect()
}


fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("小工具")
        .window_size((1200.0, 400.0));
    let initial_state = SVNAddress::new();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn convert_address(src: &String) -> String {
    let mut ret;
    if src.contains(BASE_URL) {
        ret = src.replace(BASE_URL, "softwarerepo:").trim_end_matches(|c| c == '/' || c == ' ').to_string();
        ret = format!("[{}]", ret);
    } else {
        ret = "格式错误".to_owned()
    }
    ret
}


fn build_root_widget() -> impl Widget<SVNAddress> {
    let label_svn = Label::new("点击文本框复制内容...");

    let textbox = TextBox::multiline()
        .with_placeholder("原始地址")
        .expand_width()
        .lens(SVNAddress::old);
    let textbox_name = TextBox::new()
        .with_placeholder("用户名")
        .lens(SVNAddress::name);

    let textbox_out = Flex::column()
        .with_flex_child(
            druid::widget::List::new(|| {
                TextBox::new()
                    .expand_width()
                    .lens(TextBoxData::text)
                    .on_click(
                        |_ctx, data, _env|
                            {
                                let mut clipboard = Application::global().clipboard();
                                clipboard.put_string(&data.text);
                            }
                    )
            }),
            1.0,
        );
    let btn_process = Button::<SVNAddress>::new("转换").on_click(|_ctx, _data, _env| _data.update());
    let btn_open_url = Button::<SVNAddress>::new("打开页面").on_click(|_ctx, _data, _env| {
        if let Err(e) = open::with("http://172.17.102.22:3343/csvn/repo/editAuthorization?", "chrome") {
            eprintln!("Failed to open URL: {}", e);
        }
    });
    let btn_save_local = Button::<SVNAddress>::new("保存到桌面")
        .on_click(|_ctx, _data, _env| {
            let log = _data.merged_new_addr();
            info!("{}",log);
        });

    let btn_apply_to_remote = Button::<SVNAddress>::new("应用到服务器")
        .on_click(|_ctx, _data, _env| {
            let log = _data.merged_new_addr();
            info!("{}",log);
        });
    let svn_column = Flex::column()
        .with_child(label_svn)
        .with_flex_child(textbox_out, 1.0)
        .with_default_spacer()
        .with_flex_child(textbox, 1.0)
        .with_default_spacer()
        .with_child(
            Flex::row()
                .with_child(textbox_name)
                .with_default_spacer()
                .with_child(Checkbox::new("读写").lens(SVNAddress::read_write))
                .with_default_spacer()
                .with_child(btn_process)
        )
        .with_default_spacer()
        .with_default_spacer()
        .with_child(btn_apply_to_remote)
        .with_flex_child(
            Flex::row()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .with_child(btn_open_url)
                .with_child(btn_save_local),
            FlexParams::new(1.0, CrossAxisAlignment::End),
        )
        .align_vertical(druid::UnitPoint::CENTER);


    let tabs = Tabs::new()
        .with_tab("SVN地址转换", svn_column)
        .with_tab("Proxy", Label::new("Proxy settings"));

    Flex::row()
        .with_flex_child(tabs, 1.0)
}
