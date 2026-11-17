use smol_str::SmolStr;

use super::*;
use crate::{
    component::pane::{
        block::{
            BarItem,
            Block,
            Column,
            FormRow,
            Height,
            Kind,
            Node,
            Span,
            Table,
            TableRow,
            Tone,
        },
        control::{
            Control,
            Press,
        },
        draw::Runs,
        layout::Item,
        pool::Pool,
    },
    face::measure::text,
};

fn s(text: &str) -> SmolStr {
    SmolStr::new(text)
}

fn prose(id: u32, height: Height, text: &str) -> Block {
    Block {
        id,
        height,
        kind: Kind::Text(vec![Span {
            text: s(text),
            tone: Tone::Body,
            link: None,
        }]),
    }
}

fn form(id: u32, height: Height, rows: Vec<FormRow>) -> Block {
    Block {
        id,
        height,
        kind: Kind::Form(rows),
    }
}

fn row(id: u32, label: &str, control: Control) -> FormRow {
    FormRow {
        id,
        label: s(label),
        control,
    }
}

fn pane(columns: u32, lines: Option<u32>, blocks: Vec<Block>) -> Pane {
    let mut pane = Pane::new(columns, lines).expect("a pane");
    pane.set_blocks(blocks).expect("blocks");
    pane
}

const LOREM: &str = "The quick brown fox jumps over the lazy dog and keeps running \
                     through the meadow until the light fails.";

fn many(lines: usize) -> String {
    (0..lines)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn text_wraps_to_the_pane() {
    let pane = pane(24, Some(12), vec![prose(1, Height::Share, LOREM)]);
    let block = &pane.laid().blocks[0];
    assert!(block.rows.len() > 1, "a long paragraph takes several lines");
    let room = pane.width() - 2.0 * PAD_X;
    for row in &block.rows {
        for item in &row.items {
            let Item::Text {
                text: line, width, ..
            } = item
            else {
                continue;
            };
            assert!((text::width(line) - width).abs() < 1.0e-3);
            assert!(*width <= room + 1.0e-3, "{line:?} is {width} ems in {room}");
        }
    }
}

#[test]
fn a_form_row_puts_its_label_left_and_its_control_right() {
    let pane = pane(
        40,
        Some(4),
        vec![form(
            1,
            Height::Share,
            vec![row(7, "Visible", Control::Toggle(true))],
        )],
    );
    let items = &pane.laid().blocks[0].rows[0].items;
    let (
        Item::Text {
            x: label, width, ..
        },
        Item::Control { x: control, .. },
    ) = (&items[0], &items[1])
    else {
        panic!("a label then a control: {items:?}");
    };
    assert!((label - PAD_X).abs() < 1.0e-4, "the label starts the row");
    assert!(
        label + width <= *control,
        "the control starts past the label"
    );
}

#[test]
fn a_shared_block_gets_what_the_fixed_ones_leave() {
    let pane = pane(
        24,
        Some(11),
        vec![
            prose(1, Height::Lines(2), "head"),
            prose(2, Height::Share, "a"),
            prose(3, Height::Share, "b"),
            prose(4, Height::Lines(1), "foot"),
        ],
    );
    let heights = pane
        .laid()
        .blocks
        .iter()
        .map(|block| block.lines)
        .collect::<Vec<_>>();
    assert_eq!(heights, [2, 4, 4, 1], "the first sharer takes the odd line");
    let tops = pane
        .laid()
        .blocks
        .iter()
        .map(|block| block.top)
        .collect::<Vec<_>>();
    assert_eq!(tops, [0, 2, 6, 10]);
}

#[test]
fn a_fixed_block_is_held_to_what_the_pane_has_left() {
    let pane = pane(
        24,
        Some(3),
        vec![
            prose(1, Height::Lines(2), "a"),
            prose(2, Height::Lines(5), "b"),
            prose(3, Height::Share, "c"),
        ],
    );
    let heights = pane
        .laid()
        .blocks
        .iter()
        .map(|block| block.lines)
        .collect::<Vec<_>>();
    assert_eq!(heights, [2, 1, 0]);
}

#[test]
fn a_pane_without_lines_grows_to_fit_up_to_the_limit() {
    let fits = pane(24, None, vec![prose(1, Height::Share, &many(6))]);
    assert_eq!(fits.laid().lines, 6);
    let capped = pane(24, None, vec![prose(1, Height::Share, &many(100))]);
    assert_eq!(capped.laid().lines, MAX_LINES);
    assert!(capped.laid().blocks[0].rail, "what does not fit scrolls");
}

#[test]
fn overflow_scrolls_and_clamps() {
    let mut pane = pane(24, Some(5), vec![prose(1, Height::Share, &many(12))]);
    let block = pane.laid().blocks[0].clone();
    assert!(block.rail);
    assert_eq!(block.content(), 12);

    assert!(pane.scroll_by(1, 3));
    assert_eq!(pane.scroll_of(1), 3);
    assert!(pane.scroll_by(1, 100));
    assert_eq!(pane.scroll_of(1), 7, "held to the last rows");
    assert!(!pane.scroll_by(1, 1), "already at the end");
    assert!(pane.scroll_by(1, -100));
    assert_eq!(pane.scroll_of(1), 0, "held to the first rows");

    let at_end = {
        pane.scroll_by(1, 100);
        pane.laid().blocks[0]
            .visible(pane.scroll_of(1))
            .map(|(line, _)| line)
            .collect::<Vec<_>>()
    };
    assert_eq!(at_end, [0, 1, 2, 3, 4], "the window stays full at the end");

    pane.set_blocks(vec![prose(1, Height::Share, &many(6))])
        .expect("blocks");
    assert_eq!(
        pane.scroll_of(1),
        1,
        "shrinking the content reclamps the offset"
    );
}

#[test]
fn a_rail_press_scrolls_to_where_it_points() {
    let mut pane = pane(24, Some(5), vec![prose(1, Height::Share, &many(12))]);
    assert!(pane.scroll_to(1, 1.0));
    assert_eq!(pane.scroll_of(1), 7);
    assert!(pane.scroll_to(1, 0.0));
    assert_eq!(pane.scroll_of(1), 0);
}

#[test]
fn the_bead_shrinks_with_the_content_and_stays_on_the_rail() {
    let short = scroll::thumb(0, 10, 5);
    let long = scroll::thumb(0, 100, 5);
    assert!(long.len < short.len);
    for offset in 0..=95 {
        let thumb = scroll::thumb(offset, 100, 5);
        assert!(thumb.start >= 0.0 && thumb.start + thumb.len <= 1.0 + 1.0e-5);
    }
    let whole = scroll::thumb(0, 4, 5);
    assert!(
        (whole.len - 1.0).abs() < f32::EPSILON,
        "nothing to scroll fills the rail"
    );
}

#[test]
fn a_tables_header_stays_while_its_rows_scroll() {
    let rows = (0..20)
        .map(|id| TableRow {
            id,
            cells: vec![s(&format!("row {id}"))],
            selected: false,
        })
        .collect();
    let mut pane = pane(
        24,
        Some(5),
        vec![Block {
            id:     1,
            height: Height::Share,
            kind:   Kind::Table(Table {
                columns: vec![Column {
                    title:   s("Name"),
                    columns: 12,
                }],
                rows,
            }),
        }],
    );
    pane.scroll_by(1, 6);
    let lines = pane.laid().blocks[0]
        .visible(pane.scroll_of(1))
        .map(|(line, row)| {
            let Some(Item::Text { text, .. }) = row.items.first() else {
                panic!("a cell");
            };
            (line, text.to_string())
        })
        .collect::<Vec<_>>();
    assert_eq!(lines[0], (0, "Name".to_owned()));
    assert_eq!(lines[1], (1, "row 6".to_owned()));
    assert_eq!(lines.len(), 5);
}

fn everything() -> Pane {
    let tree = (0..4)
        .map(|id| Node {
            id:       40 + id,
            label:    s(&format!("node {id}")),
            depth:    id % 3,
            open:     (id % 2 == 0).then_some(id == 0),
            selected: id == 3,
        })
        .collect();
    let table = Table {
        columns: vec![
            Column {
                title:   s("Layer"),
                columns: 14,
            },
            Column {
                title:   s("Size"),
                columns: 8,
            },
        ],
        rows:    (0..3)
            .map(|id| TableRow {
                id:       30 + id,
                cells:    vec![s(&format!("layer {id}")), s("12")],
                selected: id == 1,
            })
            .collect(),
    };
    let mut pane = pane(
        64,
        Some(30),
        vec![
            Block {
                id:     1,
                height: Height::Lines(1),
                kind:   Kind::Bar(vec![
                    BarItem {
                        id:      10,
                        control: Control::Action {
                            label:   s("Apply"),
                            primary: true,
                        },
                    },
                    BarItem {
                        id:      11,
                        control: Control::Toggle(false),
                    },
                    BarItem {
                        id:      12,
                        control: Control::Choice {
                            options:  vec![s("One"), s("Two"), s("Three")],
                            selected: 1,
                        },
                    },
                ]),
            },
            form(
                2,
                Height::Lines(8),
                vec![
                    row(20, "Name", Control::Value(s("Cube"))),
                    row(21, "On", Control::Toggle(true)),
                    row(22, "Opacity", Control::Slider(0.4)),
                    row(23, "Load", Control::Meter(0.7)),
                    row(
                        24,
                        "X",
                        Control::Number {
                            value:      1.25,
                            per_column: 0.1,
                        },
                    ),
                    row(25, "Where", Control::Link(s("Home"))),
                    row(
                        26,
                        "Mode",
                        Control::Choice {
                            options:  vec![s("A"), s("B")],
                            selected: 0,
                        },
                    ),
                    row(
                        27,
                        "Go",
                        Control::Action {
                            label:   s("Run"),
                            primary: false,
                        },
                    ),
                ],
            ),
            Block {
                id:     3,
                height: Height::Lines(4),
                kind:   Kind::Table(table),
            },
            Block {
                id:     4,
                height: Height::Lines(4),
                kind:   Kind::Tree(tree),
            },
            Block {
                id:     5,
                height: Height::Share,
                kind:   Kind::Text(vec![Span {
                    text: s("See the docs for more."),
                    tone: Tone::Muted,
                    link: Some(50),
                }]),
            },
        ],
    );
    pane.set_title("Inspector").expect("title");
    pane.set_status(Some("4 nodes")).expect("status");
    pane
}

fn centre(rect: Rect) -> Vec2 {
    Vec2::new(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
}

fn bounds(run: &draw::TextRun) -> Rect {
    Rect {
        x: run.at.x,
        y: BASELINE.mul_add(-run.scale, run.at.y),
        w: run.width,
        h: LINE * run.scale,
    }
}

fn every_targeted_run_is_hit(pane: &Pane) -> usize {
    let runs = pane.runs();
    let mut checked = 0;
    for run in &runs.text {
        let Some(id) = run.target else { continue };
        let hit = pane.hit(centre(bounds(run)));
        assert_eq!(hit.and_then(|hit| hit.id), Some(id), "text {:?}", run.text);
        checked += 1;
    }
    for run in &runs.rects {
        let Some(id) = run.target else { continue };
        let hit = pane.hit(centre(run.rect));
        assert_eq!(
            hit.and_then(|hit| hit.id),
            Some(id),
            "{:?} at {:?}",
            run.fill,
            run.rect
        );
        checked += 1;
    }
    checked
}

#[test]
fn every_controls_run_is_what_a_press_on_it_answers_for() {
    let pane = everything();
    assert!(
        every_targeted_run_is_hit(&pane) > 20,
        "the pane draws its controls"
    );
}

#[test]
fn the_runs_a_scrolled_pane_draws_are_still_what_it_hits() {
    let mut pane = pane(
        24,
        Some(4),
        vec![form(
            1,
            Height::Share,
            (0..12)
                .map(|id| {
                    row(
                        id,
                        "Row",
                        Control::Action {
                            label:   s("Go"),
                            primary: false,
                        },
                    )
                })
                .collect(),
        )],
    );
    pane.scroll_by(1, 5);
    assert!(every_targeted_run_is_hit(&pane) >= 8);
    let first = pane.runs().text.iter().find_map(|run| run.target);
    assert_eq!(first, Some(5), "the first visible row is the fifth");
}

#[test]
fn a_press_means_what_the_control_it_lands_on_means() {
    let pane = everything();
    let press = |id: u32| {
        let runs = pane.runs();
        let at = runs
            .rects
            .iter()
            .find(|run| run.target == Some(id))
            .map(|run| centre(run.rect))
            .or_else(|| {
                runs.text
                    .iter()
                    .find(|run| run.target == Some(id))
                    .map(|run| centre(bounds(run)))
            })
            .expect("a run for the control");
        pane.hit(at).expect("a hit").press
    };
    assert_eq!(press(10), Press::Act);
    assert_eq!(press(11), Press::Toggle(true));
    assert_eq!(press(24), Press::Scrub { per_column: 0.1 });
    assert_eq!(press(25), Press::Follow);
    assert_eq!(press(50), Press::Follow);
    assert_eq!(press(31), Press::Select);
    assert!(matches!(press(22), Press::Slide(_)));
    assert_eq!(press(40), Press::Fold(false));
}

#[test]
fn pressing_along_a_slider_asks_for_the_fraction_pressed() {
    let slider = Control::Slider(0.0);
    for (x, want) in [(0.0, 0.0), (5.0, 0.5), (10.0, 1.0)] {
        let Press::Slide(got) = slider.press(x, 10.0) else {
            panic!("a slide");
        };
        assert!((got - want).abs() < 1.0e-6);
    }
}

#[test]
fn pressing_an_option_of_a_choice_chooses_that_option() {
    let choice = Control::Choice {
        options:  vec![s("One"), s("Two")],
        selected: 0,
    };
    let segments = control::segments(&[s("One"), s("Two")]);
    assert_eq!(choice.press(segments[0].0 + 0.1, 20.0), Press::Choose(0));
    assert_eq!(choice.press(segments[1].0 + 0.1, 20.0), Press::Choose(1));
}

#[test]
fn a_point_off_the_body_hits_nothing() {
    let pane = everything();
    assert_eq!(pane.hit(Vec2::new(-1.0, 0.1)), None);
    let frame = pane.frame();
    assert_eq!(pane.hit(Vec2::new(1.0, frame.height + 1.0)), None);
}

#[test]
fn tab_visits_each_control_in_reading_order_and_wraps() {
    let mut pane = everything();
    let order = pane.focus_order();
    assert_eq!(&order[..3], [10, 11, 12], "the bar comes first");
    assert!(!order.contains(&20), "a value is nothing to stop on");

    let mut visited = Vec::new();
    for _ in 0..order.len() {
        visited.push(pane.focus_step(false).expect("a stop"));
    }
    assert_eq!(visited, order);
    assert_eq!(pane.focus_step(false), Some(order[0]), "wraps forward");
    assert_eq!(
        pane.focus_step(true),
        Some(*order.last().expect("stops")),
        "wraps back"
    );
}

#[test]
fn tab_scrolls_the_control_it_lands_on_into_view() {
    let mut pane = pane(
        24,
        Some(3),
        vec![form(
            1,
            Height::Share,
            (0..10)
                .map(|id| row(id, "Row", Control::Toggle(false)))
                .collect(),
        )],
    );
    for _ in 0..6 {
        pane.focus_step(false);
    }
    assert_eq!(pane.focus(), Some(5));
    assert_eq!(
        pane.scroll_of(1),
        3,
        "the sixth row is the last of three on view"
    );
    pane.focus_step(true);
    pane.focus_step(true);
    pane.focus_step(true);
    pane.focus_step(true);
    assert_eq!(pane.focus(), Some(1));
    assert_eq!(pane.scroll_of(1), 1);
}

#[test]
fn enter_does_what_a_press_on_the_focused_control_does() {
    let mut pane = everything();
    assert_eq!(pane.activate(), None, "nothing has the keyboard");
    pane.set_focus(Some(10));
    assert_eq!(pane.activate(), Some(Event::Pressed(10)));
    pane.set_focus(Some(11));
    assert_eq!(pane.activate(), Some(Event::Toggled(11, true)));
    pane.set_focus(Some(12));
    assert_eq!(pane.activate(), Some(Event::Chosen(12, 2)));
    pane.set_focus(Some(31));
    assert_eq!(pane.activate(), Some(Event::Selected(31)));
}

#[test]
fn the_wrong_width_or_height_is_refused() {
    assert_eq!(Pane::new(30, None).err(), Some(Refused::Width(30)));
    assert_eq!(Pane::new(24, Some(0)).err(), Some(Refused::Lines(0)));
    assert_eq!(Pane::new(24, Some(41)).err(), Some(Refused::Lines(41)));
}

#[test]
fn too_many_blocks_rows_or_bytes_are_refused_and_the_old_blocks_kept() {
    let mut pane = pane(24, Some(4), vec![prose(1, Height::Share, "kept")]);
    let blocks = (0..=MAX_BLOCKS as u32)
        .map(|id| prose(id, Height::Share, "x"))
        .collect();
    assert_eq!(
        pane.set_blocks(blocks),
        Err(Refused::Blocks(MAX_BLOCKS + 1))
    );
    let long = "x".repeat(MAX_TEXT_BYTES + 1);
    assert_eq!(
        pane.set_blocks(vec![prose(1, Height::Share, &long)]),
        Err(Refused::Text(MAX_TEXT_BYTES + 1))
    );
    let rows = (0..=MAX_ROWS as u32)
        .map(|id| row(id, "r", Control::Toggle(false)))
        .collect();
    assert_eq!(
        pane.set_blocks(vec![form(1, Height::Share, rows)]),
        Err(Refused::Rows(MAX_ROWS + 1))
    );
    assert_eq!(pane.laid().blocks.len(), 1);
    assert_eq!(
        pane.set_blocks(vec![prose(1, Height::Share, "kept")]),
        Ok(false)
    );
}

#[test]
fn a_frame_holds_the_title_above_the_body_and_the_status_below() {
    let plain = pane(24, Some(4), vec![]);
    let mut framed = pane(24, Some(4), vec![]);
    framed.set_title("Title").expect("title");
    framed.set_status(Some("status")).expect("status");
    assert!(framed.frame().body_top > plain.frame().body_top);
    assert!(framed.frame().height > plain.frame().height);
    let status = framed.frame().status_top.expect("a status line");
    assert!(status >= framed.frame().body_top + 4.0 * LINE);

    let runs = framed.runs();
    let texts = runs
        .text
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Title", "status"]);
}

fn writes(pool: &mut Pool<(draw::TextRun, u8)>, runs: &Runs) -> usize {
    pool.plan(runs.text.iter().map(|run| (run.clone(), 8)).collect())
        .writes
        .len()
}

#[test]
fn an_unchanged_pane_writes_nothing_and_a_changed_line_writes_one_prim() {
    let mut pane = pane(
        24,
        Some(12),
        vec![form(
            1,
            Height::Share,
            (0..6)
                .map(|id| row(id, "Row", Control::Value(s("same"))))
                .collect(),
        )],
    );
    let mut pool = Pool::default();
    let first = writes(&mut pool, &pane.runs());
    assert_eq!(first, 12, "a label and a value for each row");
    assert_eq!(
        writes(&mut pool, &pane.runs()),
        0,
        "at rest it writes nothing"
    );

    pane.set_blocks(vec![form(
        1,
        Height::Share,
        (0..6)
            .map(|id| {
                let value = if id == 3 { "changed" } else { "same" };
                row(id, "Row", Control::Value(s(value)))
            })
            .collect(),
    )])
    .expect("blocks");
    assert_eq!(writes(&mut pool, &pane.runs()), 1, "only the changed line");
}

#[test]
fn hover_redraws_only_the_marks() {
    let mut pane = everything();
    let mut rects = Pool::default();
    let plan = |pool: &mut Pool<(draw::RectRun, u8)>, pane: &Pane| {
        pool.plan(
            pane.runs()
                .rects
                .iter()
                .map(|run| (run.clone(), 8))
                .collect(),
        )
    };
    plan(&mut rects, &pane);
    assert!(plan(&mut rects, &pane).writes.is_empty());
    pane.set_hovered(Some(21));
    let marks = 3;
    assert!(
        plan(&mut rects, &pane).writes.len() <= marks,
        "a hover moves the marks after the pane and nothing before them"
    );
}

#[test]
fn an_eighty_by_forty_pane_of_text_writes_one_prim_per_line_at_most() {
    let spans = (0..60)
        .map(|n| Span {
            text: s(&format!(
                "Paragraph {n} of the page, long enough to take a line or two of an eighty column pane."
            )),
            tone: Tone::Body,
            link: None,
        })
        .collect();
    let mut pane = pane(
        80,
        Some(40),
        vec![Block {
            id:     1,
            height: Height::Share,
            kind:   Kind::Text(spans),
        }],
    );
    pane.set_title("Notes").expect("title");
    let runs = pane.runs();
    let mut pool = Pool::default();
    let first = writes(&mut pool, &runs);
    assert_eq!(first, 41, "forty lines and the title");
    assert_eq!(writes(&mut pool, &runs), 0);
    pane.scroll_by(1, 1);
    assert!(
        writes(&mut pool, &pane.runs()) <= 41,
        "a scroll rewrites the lines, never more"
    );
}
