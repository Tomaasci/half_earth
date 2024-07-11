use std::time::Duration;

use leptos::*;

use crate::{
    anim::animation,
    util::{card_scale, to_ws_el},
};

pub fn shake_screen() {
    document().body().map(|body| {
        // TODO
        // window.audioManager.playOneShot('/assets/sounds/impact.mp3');
        body.class_list().add_1("shake").unwrap();
        set_timeout(
            move || {
                body.class_list().remove_1("shake").unwrap();
            },
            Duration::from_millis(500),
        );
    });
}

pub fn shake_progress(elem: web_sys::HtmlElement) {
    if let Some(elem) = elem.parent_element() {
        elem.class_list().add_2("scan-error", "shake");
        set_timeout(
            move || {
                elem.class_list().remove_2("scan-error", "shake").unwrap();
            },
            Duration::from_millis(350),
        );
    }
}

pub fn pulse_card() {
    if let Some(elem) = document().query_selector(".draggable.active").unwrap() {
        let from = card_scale();
        let to = from * 1.05;
        // animation([from],[to], 100., Some(||))

        // TODO
        // animate(consts.cardScale, consts.cardScale*1.05, 100, (val) => {
        //   updateTransform(el, {scale: val});
        // }, () => {
        //   animate(consts.cardScale*1.05, consts.cardScale, 100, (val) => {
        //     updateTransform(el, {scale: val});
        //   });
        // });
    }
}

pub fn shrink_pulse_card() {
    if let Some(elem) = document().query_selector(".draggable.active").unwrap() {
        let from = card_scale();
        let to = from * 0.95;
        // TODO
        // animate(consts.cardScale, consts.cardScale*0.95, 100, (val) => {
        //   updateTransform(el, {scale: val});
        // }, () => {
        //   animate(consts.cardScale*0.95, consts.cardScale, 100, (val) => {
        //     updateTransform(el, {scale: val});
        //   });
        // });
    }
}

pub fn pulse_level() {
    if let Some(elem) = document()
        .query_selector(".draggable.active .project-cost")
        .unwrap()
    {
        // TODO
        // animate(1, 1.2, 200, (val) => {
        //   el.style.transform = `scale(${val})`;
        // }, () => {
        //   animate(1.2, 1, 200, (val) => {
        //     el.style.transform = `scale(${val})`;
        //   });
        // });
    }
}
