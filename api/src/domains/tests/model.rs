use super::*;

fn page_query(id: Option<u64>, limit: Option<usize>) -> PageQuery {
    PageQuery { id, limit }
}

fn rows(count: u64) -> Vec<u64> {
    (0..count).map(|i| 100 - i).collect()
}

#[test]
fn page_size_defaults_when_no_limit_is_given() {
    assert_eq!(page_query(None, None).page_size(), DEFAULT_PAGE_SIZE);
}

#[test]
fn page_size_is_clamped_to_the_allowed_range() {
    assert_eq!(page_query(None, Some(0)).page_size(), 1);
    assert_eq!(page_query(None, Some(25)).page_size(), 25);
    assert_eq!(
        page_query(None, Some(MAX_PAGE_SIZE + 1)).page_size(),
        MAX_PAGE_SIZE
    );
}

#[test]
fn fetch_size_asks_for_one_row_beyond_the_page() {
    assert_eq!(page_query(None, Some(25)).fetch_size(), 26);
}

#[test]
fn a_full_fetch_yields_a_cursor_on_the_last_returned_row() {
    let page = Page::build(rows(11), 10, |row| *row);

    assert_eq!(page.items.len(), 10);
    assert_eq!(page.items.last(), Some(&91));
    assert_eq!(page.next_page_params, Some(NextPageParams { id: 91 }));
}

#[test]
fn an_exact_fetch_has_no_next_page() {
    let page = Page::build(rows(10), 10, |row| *row);

    assert_eq!(page.items.len(), 10);
    assert_eq!(page.next_page_params, None);
}

#[test]
fn a_short_fetch_has_no_next_page() {
    let page = Page::build(rows(3), 10, |row| *row);

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.next_page_params, None);
}

#[test]
fn an_empty_fetch_has_no_items_and_no_next_page() {
    let page = Page::build(rows(0), 10, |row| *row);

    assert!(page.items.is_empty());
    assert_eq!(page.next_page_params, None);
}

#[test]
fn paging_through_a_cursor_does_not_skip_a_row() {
    let all = rows(21);

    let first = Page::build(all.clone(), 10, |row| *row);
    let cursor = first.next_page_params.unwrap().id;

    let remaining: Vec<u64> = all.into_iter().filter(|row| *row < cursor).collect();
    let second = Page::build(remaining, 10, |row| *row);

    assert_eq!(first.items.last(), Some(&91));
    assert_eq!(second.items.first(), Some(&90));
    assert_eq!(second.items.last(), Some(&81));
    assert_eq!(second.next_page_params, Some(NextPageParams { id: 81 }));
}
