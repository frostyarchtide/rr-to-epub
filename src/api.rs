use crate::{cache::Cache, epub::Book, GlobalArgs};

pub struct RoyalRoadApi;

impl RoyalRoadApi {
    pub fn new() -> Self {
        Self
    }
    pub async fn get_book(&self, id: u32, global_args: &GlobalArgs) -> eyre::Result<Book> {
        // Do the initial metadata fetch of the book.
        let mut book = Book::new(id).await?;

        // Update the cover.
        tracing::info!("Updating cover.");
        book.update_cover().await?;

        // Check the cache.
        let cached = Cache::read_book(id)?;
        match cached {
            Some(mut cached) => {
                cached.cover_url = book.cover_url;

                // Compare cached and fetched to see if any chapters are out-of-date or missing
                // content.
                for (i, chapter) in book.chapters.iter().enumerate() {
                    if let Some(cached_chapter) = cached.chapters.iter().find(|c| c.url == chapter.url) {
                        if cached_chapter.date != chapter.date || cached_chapter.content.is_none() {
                            cached.update_chapter_content(i, global_args).await?;
                            Cache::write_book(&cached)?;
                        }
                    } else {
                        cached.chapters.push(chapter.clone());
                        cached.update_chapter_content(i, global_args).await?;
                        Cache::write_book(&cached)?;
                    }
                }

                Ok(cached)
            }
            None => {
                // Load book chapters and cache book.
                for i in 0..book.chapters.len() {
                    book.update_chapter_content(i, global_args).await?;
                    Cache::write_book(&book)?;
                }

                // Return book.
                Ok(book)
            }
        }
    }
}
