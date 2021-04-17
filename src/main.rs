use std::cmp::{max, min};
use xcb::shape;

//绘制矩形
fn set_shape(conn: &xcb::Connection, window: xcb::Window, rects: &[xcb::Rectangle]) {
    shape::rectangles(
        &conn,
        shape::SO_SET as u8,
        shape::SK_BOUNDING as u8,
        0,
        window,
        0,
        0,
        &rects,
    );
    conn.flush();
}

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window = conn.generate_id();

    let values = [
        // TODO 按键处理
        (xcb::CW_BACK_PIXEL, 0x00_00_00_00), //线条的16进制rgb颜色
        (
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS,
        ), //TODO 以后处理
        (xcb::CW_OVERRIDE_REDIRECT, 1 as u32), // 不要被window manager接管
    ];

    xcb::create_window(
        &conn,
        xcb::COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        0,                         // x
        0,                         // y
        screen.width_in_pixels(),  // width
        screen.height_in_pixels(), // height
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );

    let title = "slurp-x";

    xcb::change_property(
        &conn,
        xcb::PROP_MODE_REPLACE as u8,
        window,
        xcb::ATOM_WM_NAME,
        xcb::ATOM_STRING,
        8,
        title.as_bytes(),
    );

    let font = conn.generate_id();
    xcb::open_font(&conn, font, "cursor");

    //创建光标
    let cursor = conn.generate_id();
    xcb::create_glyph_cursor(&conn, cursor, font, font, 0, 35, 0, 0, 0, 0, 0, 0);

    xcb::grab_pointer(
        &conn,
        true,
        screen.root(),
        (xcb::EVENT_MASK_BUTTON_RELEASE
            | xcb::EVENT_MASK_BUTTON_PRESS
            | xcb::EVENT_MASK_BUTTON_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::NONE,
        cursor,
        xcb::CURRENT_TIME,
    )
    .get_reply()
    .unwrap();

    set_shape(&conn, window, &[xcb::Rectangle::new(0, 0, 0, 0)]); //初始化设置默认值为空

    xcb::map_window(&conn, window);

    conn.flush();

    let (mut start_x, mut start_y, mut width, mut height) = (0, 0, 0, 0);
    let (mut x, mut y);

    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev.response_type() {
            xcb::BUTTON_PRESS => {
                let button_press: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&ev) };

                if button_press.detail() == 3 {
                    println!("exit");
                    return;
                }

                start_x = button_press.event_x();
                start_y = button_press.event_y();
            }
            xcb::KEY_PRESS => {
                println!("Exiting due to key press");
                return;
            }
            xcb::MOTION_NOTIFY => {
                let motion: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&ev) };
                x = motion.event_x();
                y = motion.event_y();

                // TODO 不知道性能有没有问题，先糊上
                let top_x = min(x, start_x);
                let top_y = min(y, start_y);
                let bot_x = max(x, start_x);
                let bot_y = max(y, start_y);

                width = (x - start_x).abs() as u16;
                height = (y - start_y).abs() as u16;

                let rects = [
                    xcb::Rectangle::new(top_x, top_y, 2, height),
                    xcb::Rectangle::new(top_x, top_y, width, 2),
                    xcb::Rectangle::new(bot_x, top_y, 2, height),
                    xcb::Rectangle::new(top_x, bot_y, width, 2), //2是线条宽度
                ];
                set_shape(&conn, window, &rects); //绘制矩形
            }
            xcb::BUTTON_RELEASE => {
                let motion: &xcb::ButtonReleaseEvent = unsafe { xcb::cast_event(&ev) };
                match motion.detail() {
                    5 => continue, // TODO 向下滚轮处理
                    4 => continue, // TODO 向上滚处理
                    _ => break,    // 释放鼠标后继续
                }
            }
            _ => continue,
        };
    }

    println!("{}{}{}{}", start_x, start_y, width, height);
}
