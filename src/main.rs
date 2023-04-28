// 目标：写一个程序，将平时的SVN地址格式转换一下

#![windows_subsystem = "windows"]

mod tests;

use druid::widget::{Flex, Label, Tabs};
use druid::{
    widget::{Button, TextBox},
    AppLauncher, Data, Lens, Widget, WidgetExt, WindowDesc,
};
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

const BASE_URL: &str = "http://172.17.102.22:18080/svn/softwarerepo";

#[derive(Debug, Data, Clone, Lens)]
struct SVNAddress {
    old: String,
    #[data(eq)]

    new_addr: Vec<String>,
    new_addr_display:String,
    name: String,
}


impl SVNAddress {
    fn new() -> SVNAddress {
        SVNAddress {
            old: "".to_string(),
            new_addr: vec!["".to_string()],
            new_addr_display: "".to_string(),
            name: "t".to_string(),
        }
    }
    fn update(&mut self) {
        self.new_addr.clear();
        let srcs = extract_substrings_containing_base_url(self.old.as_str());
        if srcs.is_empty() || !srcs.first().unwrap().starts_with("http") {
            self.new_addr.push("格式错误".to_owned());
        }
        for x in srcs {
            self.new_addr.push(convert_address(&x));
        }
        self.new_addr_display = self.new_addr.join("\n");
        update_clipboard(self.new_addr_display.to_owned());
    }
}


fn extract_substrings_containing_base_url(input_str: &str) -> Vec<String> {
    input_str
        .split_whitespace()
        .filter(|s| s.contains(BASE_URL))
        .map(|s| s.to_string())
        .collect()
}

fn update_clipboard(content: String) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    println!("{:?}", ctx.get_contents());
    ctx.set_contents(content).unwrap();
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

#[derive(Data, Clone, Lens)]
struct AppState {
    name: String,
}


fn build_root_widget() -> impl Widget<SVNAddress> {
    let label_svn = Label::new("SVN 地址转换：");

    let textbox = TextBox::multiline()
        .with_placeholder("原始地址")
        .expand_width()
        .lens(SVNAddress::old);

    let textbox_out = TextBox::multiline()
        .with_placeholder("目标地址")
        .expand_width()
        .lens(SVNAddress::new_addr_display);

    let button1 = Button::<SVNAddress>::new("转换").on_click(|_ctx, _data, _env| _data.update());
    let btn_open_url = Button::<SVNAddress>::new("打开页面").on_click(|_ctx, _data, _env| {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        println!("{:?}", ctx.get_contents());
        ctx.set_contents(_data.new_addr.join("\n")).unwrap();
        if let Err(e) = open::with("http://172.17.102.22:3343/csvn/repo/editAuthorization?","chrome") {
            eprintln!("Failed to open URL: {}", e);
        }
    });
    let svn_column = Flex::column()
        .with_child(label_svn)
        .with_flex_child(textbox_out, 1.0)
        .with_default_spacer()
        .with_flex_child(textbox, 1.0)
        .with_default_spacer()
        .with_child(button1)
        .with_child(btn_open_url)
        .align_vertical(druid::UnitPoint::CENTER);

    let tabs = Tabs::new()
        .with_tab("SVN地址转换", svn_column)
        .with_tab("Proxy", Label::new("Proxy settings"));

    Flex::row()
        .with_flex_child(tabs, 1.0)
}
