use std::num::NonZeroU32;

#[allow(clippy::upper_case_acronyms)]
type ASOffset = u32;
#[allow(clippy::upper_case_acronyms)]
type ASVersion = NonZeroU32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnimationId {
    offset: ASOffset,
    version: ASVersion,
}

impl AnimationId {
    pub fn new(offset: ASOffset, version: NonZeroU32) -> Self {
        AnimationId { offset, version }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum ASEntry<Value> {
    Busy(ASVersion, Value),
    Free(ASVersion, ASOffset), // next free. Free entries form a linked list.
    LastFree(ASVersion),
}

#[derive(Debug)]
// This is a basic slot map
pub(in crate::animation) struct AnimationStorage<Value> {
    contents: Vec<ASEntry<Value>>,
    size: ASOffset,
    first_free: Option<ASOffset>,
}

//Derive creates an incorrect constraint
impl<Value> Default for AnimationStorage<Value> {
    fn default() -> Self {
        log::info!("Create");
        AnimationStorage {
            contents: Default::default(),
            size: Default::default(),
            first_free: Default::default(),
        }
    }
}

impl<Value> AnimationStorage<Value> {
    // O(n)
    pub(in crate::animation) fn remove_if(
        &mut self,
        mut f: impl FnMut(AnimationId, &mut Value) -> bool,
    ) {
        for (offset, entry) in self.contents.iter_mut().enumerate() {
            let offset = offset as ASOffset;
            let (version, remove) = match entry {
                ASEntry::Busy(version, value) => {
                    (*version, f(AnimationId::new(offset, *version), value))
                }
                ASEntry::Free(version, _) | ASEntry::LastFree(version) => (*version, false),
            };

            if remove {
                *entry = self
                    .first_free
                    .map(|next_free| ASEntry::Free(version, next_free))
                    .unwrap_or_else(|| ASEntry::LastFree(version));
                self.first_free = Some(offset);
                self.size -= 1;
            }
        }
    }

    pub(in crate::animation) fn size(&self) -> ASOffset {
        self.size
    }

    pub(in crate::animation) fn is_empty(&self) -> bool {
        self.size == 0
    }

    // O(1)
    pub(in crate::animation) fn insert(&mut self, value: Value) -> AnimationId {
        self.size += 1;
        if let Some(offset) = self.first_free.take() {
            let entry = &mut self.contents[offset as usize];
            let (first_free, version) = match entry {
                ASEntry::LastFree(version) => (None, version),
                ASEntry::Free(version, next_free) => (Some(*next_free), version),
                ASEntry::Busy(..) => panic!("Free list pointing to busy entry"),
            };
            self.first_free = first_free;
            let version = NonZeroU32::new(version.get().wrapping_add(1).max(1)).unwrap();
            *entry = ASEntry::Busy(version, value);
            AnimationId::new(offset, version)
        } else {
            let version = NonZeroU32::new(1).unwrap();
            let id = AnimationId::new(self.contents.len() as u32, version);
            self.contents.push(ASEntry::Busy(version, value));
            id
        }
    }

    // O(1)
    pub(in crate::animation) fn contains(&self, id: AnimationId) -> bool {
        id.offset < self.contents.len() as u32
            && matches!(self.contents[id.offset as usize], ASEntry::Busy(version, _) if version == id.version)
    }

    // O(1)
    pub(in crate::animation) fn get(&self, id: AnimationId) -> Option<&Value> {
        self.contents
            .get(id.offset as usize)
            .and_then(|entry| match entry {
                ASEntry::Busy(version, seg) if *version == id.version => Some(seg),
                _ => None,
            })
    }

    pub(in crate::animation) fn get_mut(&mut self, id: AnimationId) -> Option<&mut Value> {
        self.contents
            .get_mut(id.offset as usize)
            .and_then(|entry| match entry {
                ASEntry::Busy(version, seg) if *version == id.version => Some(seg),
                _ => None,
            })
    }

    /*
    pub(in crate::animation) fn iter(&self) -> impl Iterator<Item = &Value> {
        self.contents.iter().flat_map(|content| match content {
            ASEntry::Busy(_, seg) => Some(seg),
            _ => None,
        })
    }

    pub(in crate::animation) fn clear(&mut self) {
        self.contents.clear();
        self.size = Default::default();
        self.first_free = Default::default();
    }
     */
}
