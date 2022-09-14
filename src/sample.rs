/// A generic wrapper around any type `T` that can be considered as being a sample test vector.
/// It has associated `label` that described the sample and validity flag (`valid`)
/// indicating whether the sample is correct - i.e. whether it is a valid CasperNetwork transaction.
#[derive(Debug, Clone)]
pub(crate) struct Sample<V> {
    label: String,
    sample: V,
    valid: bool,
}

impl<V> Sample<V> {
    pub(crate) fn new<S: Into<String>>(label: S, sample: V, valid: bool) -> Sample<V> {
        Sample {
            label: label.into(),
            sample,
            valid,
        }
    }

    pub(crate) fn destructure(self) -> (String, V, bool) {
        (self.label, self.sample, self.valid)
    }

    pub(crate) fn add_label(&mut self, label: String) {
        self.label = format!("{}__{}", self.label, label);
    }
}
