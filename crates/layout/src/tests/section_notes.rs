use super::*;
use wordcraft_doc::props::LineSpacing;

/// A short note fits below the last real paragraph; only the closing mark exceeds its area.
fn section_note_doc(closing: Option<&str>, numbered: bool) -> (Document, u32) {
    let text = closing.map_or_else(|| "Intro\nFill\nAAA\nNext".to_owned(), |mark| format!("Intro\nFill\nAAA\n{mark}\nNext"));
    let mut d = pagination_fixture_document(&text);
    d.last_section.page_h = 300.0;
    d.last_section.margin_top = 30.0;
    d.last_section.margin_bottom = 30.0;
    d.para_mut(StoryRef::Body, &Path::top(1)).unwrap().props.space_before = Some(148.0);
    if numbered {
        let num = d.numbering.add_list(wordcraft_doc::ListKind::Numbered);
        for block in [1, 2] {
            d.para_mut(StoryRef::Body, &Path::top(block)).unwrap().props.numbering = Some(wordcraft_doc::props::NumRef { num, level: 0 });
        }
    }
    let next = if closing.is_some() { 4 } else { 3 };
    d.para_mut(StoryRef::Body, &Path::top(next)).unwrap().props.page_break_before = Some(true);
    if closing.is_some() {
        let section = d.last_section.clone();
        d.para_mut(StoryRef::Body, &Path::top(3)).unwrap().section = Some(Box::new(section));
    }
    let mut note = Paragraph::with_text("Note", Default::default());
    note.props.line_spacing = Some(LineSpacing::Exactly(14.0));
    note.props.space_before = Some(0.0);
    note.props.space_after = Some(0.0);
    let id = d.add_part(wordcraft_doc::PartKind::Footnote, vec![wordcraft_doc::para_block(note)]);
    d.insert_object(
        &Pos::body(0, 5),
        InlineObject::NoteRef { kind: wordcraft_doc::para::NoteKind::Footnote, id, custom: String::new() },
        &Default::default(),
    )
    .unwrap();
    (d, id)
}

/// #3974: retaining an empty section mark must preserve the short note's reserved area.
#[test]
fn empty_section_mark_does_not_push_footnotes_below_the_margin() {
    for numbered in [false, true] {
        let (control, control_id) = section_note_doc(None, numbered);
        let baseline = lay(&control);
        assert_eq!(baseline.pages.len(), 2);
        let expected = baseline.caret(&control.start_of(StoryRef::Part(control_id))).unwrap();
        let (d, id) = section_note_doc(Some(""), numbered);
        let l = lay(&d);
        assert_eq!(l.pages.len(), 2, "the retained mark must not create an extra page");
        let mark = l.caret(&Pos::body(3, 0)).unwrap();
        assert_eq!(mark.page, 0, "the closing mark's caret stays on the preceding page");
        assert!(mark.height > 0.0, "retaining the mark must preserve its caret height");
        let actual = l.caret(&d.start_of(StoryRef::Part(id))).unwrap();
        assert_eq!(actual.page, expected.page);
        assert!((actual.top - expected.top).abs() < 0.01, "the mark shifted the note: {actual:?}, expected {expected:?}");
        let page = &l.pages[0];
        for it in &page.items {
            if let Placed::Lines { story: StoryRef::Part(part), para, l0, l1, y, .. } = it
                && *part == id
            {
                assert!(item_bottom(*y, para, *l0, *l1).unwrap() <= page.body.bottom() + 0.01, "the note crosses the bottom margin");
            }
        }
        let separator = |page: &Page| {
            page.items.iter().find_map(|it| match it {
                Placed::Rule { x0, x1, y0, y1, .. } if (*x1 - *x0 - NOTE_SEP_W).abs() < 0.01 && (*y0 - *y1).abs() < 0.01 => Some(*y0),
                _ => None,
            })
        };
        assert!(separator(page).is_some(), "the short note has a separator");
        assert_eq!(separator(page), separator(&baseline.pages[0]), "the separator must stay in place too");
    }
}

/// Ordinary empty and text-bearing closing paragraphs still need room on their own page.
#[test]
fn ordinary_paragraphs_still_fit_above_footnotes() {
    for text in ["", "Visible"] {
        let (mut d, id) = section_note_doc(Some(text), false);
        if text.is_empty() {
            d.para_mut(StoryRef::Body, &Path::top(3)).unwrap().section = None;
        }
        let l = lay(&d);
        assert_eq!(l.pages.len(), 3, "ordinary paragraph {text:?} moves before Next's page break");
        assert_eq!(l.caret(&Pos::body(3, 0)).unwrap().page, 1);
        assert_eq!(l.caret(&d.start_of(StoryRef::Part(id))).unwrap().page, 0);
    }
}
