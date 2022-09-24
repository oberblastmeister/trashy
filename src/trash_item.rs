use either::Either;
use trash::TrashItem;

use crate::utils::swap;

pub struct MaybeIndexedTrashItems(pub Either<Vec<TrashItem>, Vec<(u32, TrashItem)>>);

impl MaybeIndexedTrashItems {
    pub fn items(self) -> impl DoubleEndedIterator<Item = TrashItem> {
        self.0.map_right(|v| v.into_iter().map(|(_, t)| t)).into_iter().into_iter()
    }

    pub fn indexed_items(
        &self,
    ) -> impl DoubleEndedIterator<Item = (u32, &'_ TrashItem)> + ExactSizeIterator {
        self.0
            .as_ref()
            .map_left(|v| v.iter().zip(0..v.len() as u32).map(swap))
            .map_right(|v| v.iter().map(|(i, t)| (*i, t)))
            .into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().either(|v| v.len(), |v| v.len())
    }
}
