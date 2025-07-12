use strum::FromRepr;

#[derive(Debug, Default, Copy, Clone, FromRepr)]
#[repr(u8)]
pub(super) enum SelectedBlock {
    #[default]
    List,
    Content,
}

impl SelectedBlock {
    pub(super) fn move_by(self, amount: i8) -> Self {
        let current_block_i = self as u8;
        SelectedBlock::from_repr(
            current_block_i
                .checked_add_signed(amount)
                .unwrap_or_default(),
        )
        .unwrap_or(self)
    }

    pub(super) fn next(self) -> Self {
        self.move_by(1)
    }

    pub(super) fn prev(self) -> Self {
        self.move_by(-1)
    }
}
