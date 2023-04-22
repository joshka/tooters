// EntityPage is a stateful struct that handles loading and displaying
// multiple pages of entities. Each page has zero or more items, and
// zero or one next and previous pages. The next and previous pages
// are loaded on demand, and the items are stored in a Vec.
// At any time, the user should be able to scroll from the top of the
// cuurent page to the previous page by scrolling up, and from the
// bottom of the current page to the next page by scrolling down.
// The EntityPage struct handles all of this, and provides a simple
// interface for the user to interact with.

use std::collections::BTreeMap;

use url::Url;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Item {
    value: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Page {
    items: Vec<Item>,
    next: Option<Url>,
    prev: Option<Url>,
}

#[allow(dead_code)]
impl Page {
    fn new(items: Vec<Item>, next: Option<Url>, prev: Option<Url>) -> Self {
        Self { items, next, prev }
    }
}

#[allow(dead_code)]
struct Pages {
    pages: BTreeMap<i32, Page>,
    current_page: usize,
}

#[allow(dead_code)]
impl Pages {
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            current_page: 0,
        }
    }

    pub fn iter_items_after(&self, page_index: i32, index: usize) -> impl Iterator<Item = &Item> {
        self.pages
            .iter()
            .skip_while(move |kv| *kv.0 < page_index)
            .flat_map(|kv| kv.1.items.iter())
            .skip(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_items_after() {
        let mut pages = Pages::new();
        pages.pages.insert(
            -1,
            Page::new(
                vec![Item { value: 1 }, Item { value: 2 }],
                Some(Url::parse("http://example.com/next").unwrap()),
                Some(Url::parse("http://example.com/prev").unwrap()),
            ),
        );
        pages.pages.insert(
            0,
            Page::new(
                vec![Item { value: 3 }, Item { value: 4 }],
                Some(Url::parse("http://example.com/next").unwrap()),
                Some(Url::parse("http://example.com/prev").unwrap()),
            ),
        );
        pages.pages.insert(
            1,
            Page::new(
                vec![Item { value: 5 }, Item { value: 6 }],
                Some(Url::parse("http://example.com/next").unwrap()),
                Some(Url::parse("http://example.com/prev").unwrap()),
            ),
        );

        let mut iter = pages.iter_items_after(0, 0);
        assert_eq!(iter.next().unwrap().value, 3);
        assert_eq!(iter.next().unwrap().value, 4);
        assert_eq!(iter.next().unwrap().value, 5);
        assert_eq!(iter.next().unwrap().value, 6);
    }
}
