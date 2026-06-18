use crate::error::ListEventsError;
use crate::error::ListMarketsError;
use polymarket_types::PaginationCursor;

#[derive(Clone, Debug)]
pub struct Page<T> {
    pub items: T,
    pub has_more: bool,
    pub next_cursor: Option<PaginationCursor>,
}

pub struct Paginator<T, E> {
    fetch: Box<dyn FnMut(Option<PaginationCursor>) -> FetchFuture<T, E> + Send>,
    cursor: Option<PaginationCursor>,
    started: bool,
    done: bool,
}

pub type FetchFuture<T, E> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Page<T>, E>> + Send>>;

impl<T, E> Paginator<T, E> {
    pub fn new<F>(fetch: F, initial_cursor: Option<PaginationCursor>) -> Self
    where
        F: FnMut(Option<PaginationCursor>) -> FetchFuture<T, E> + Send + 'static,
    {
        Self {
            fetch: Box::new(fetch),
            cursor: initial_cursor,
            started: false,
            done: false,
        }
    }

    pub async fn first_page(&mut self) -> Result<Page<T>, E> {
        self.started = true;
        let page = (self.fetch)(self.cursor.clone()).await?;
        if page.has_more {
            self.cursor.clone_from(&page.next_cursor);
        } else {
            self.done = true;
        }
        Ok(page)
    }

    pub fn from_cursor(
        cursor: Option<PaginationCursor>,
        fetch: impl FnMut(Option<PaginationCursor>) -> FetchFuture<T, E> + Send + 'static,
    ) -> Self {
        Self::new(fetch, cursor)
    }

    pub async fn next_page(&mut self) -> Result<Option<Page<T>>, E> {
        if self.done {
            return Ok(None);
        }
        if !self.started {
            return Ok(Some(self.first_page().await?));
        }
        let page = (self.fetch)(self.cursor.clone()).await?;
        if page.has_more {
            self.cursor.clone_from(&page.next_cursor);
        } else {
            self.done = true;
        }
        Ok(Some(page))
    }

    pub async fn collect_all(&mut self) -> Result<Vec<Page<T>>, E> {
        let mut pages = Vec::new();
        pages.push(self.first_page().await?);
        while let Some(page) = self.next_page().await? {
            pages.push(page);
        }
        Ok(pages)
    }
}

pub type ListMarketsPaginator =
    Paginator<Vec<polymarket_bindings::gamma::Market>, ListMarketsError>;
pub type ListEventsPaginator = Paginator<Vec<polymarket_bindings::gamma::Event>, ListEventsError>;
