// 目标：写一个程序，将平时的SVN地址格式转换一下

#![windows_subsystem = "windows"]

use druid::widget::Flex;
use druid::{
    widget::{Button, TextBox},
    AppLauncher, Data, Lens, Widget, WidgetExt, WindowDesc,
};

#[derive(Debug, Data, Clone, Lens)]
struct SVNAddress {
    old: String,
    new: String,
}

impl SVNAddress {
    fn update(&mut self) {
        self.new = convert_address(&self.old);
    }
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("SVN 地址转换")
        .window_size((400.0, 400.0));
    let initial_state: SVNAddress = SVNAddress {
        old: "".into(),
        new: "".into(),
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn convert_address(src: &String) -> String {
    let header = "http://172.17.102.22:18080/svn/softwarerepo";
    let mut ret;
    if src.contains(header) {
        ret = src.replace(header, "softwarerepo:").trim_end_matches(|c| c == '/' || c == ' ').to_string();
        ret = format!("[{}]", ret);
    } else {
        ret = "格式错误".to_owned()
    }
    ret
}

fn build_root_widget() -> impl Widget<SVNAddress> {
    // let new_text = Label::new(|data: &SVNAddress, _env: &Env| {
    //     if data.old.is_empty() {
    //         "Hello anybody!?".to_string()
    //     } else {
    //         convert_address(&data.old)
    //     }
    // });

    let textbox = TextBox::multiline()
        .with_placeholder("原始地址")
        .expand_width()
        .lens(SVNAddress::old);

    let textbox_out = TextBox::multiline()
        .with_placeholder("目标地址")
        .expand_width()
        .lens(SVNAddress::new);

    let button1 = Button::<SVNAddress>::new("转换").on_click(|_ctx, _data, _env| _data.update());
    Flex::column()
        .with_flex_child(textbox_out, 1.0)
        .with_default_spacer()
        .with_flex_child(textbox, 1.0)
        .with_default_spacer()
        .with_child(button1)
        .align_vertical(druid::UnitPoint::CENTER)
}
