pub(crate) struct Sample<V> {
    label: String,
    sample: V,
}

impl<V> Sample<V> {
    pub(crate) fn new(label: String, sample: V) -> Sample<V> {
        Sample { label, sample }
    }

    pub(crate) fn destructure(self) -> (String, V) {
        (self.label, self.sample)
    }

    pub(crate) fn map_sample<VV, F: FnOnce(V) -> VV>(self, f: F) -> Sample<VV> {
        Sample {
            label: self.label,
            sample: f(self.sample),
        }
    }

    pub(crate) fn add_label(&mut self, label: String) {
        self.label = format!("{}-{}", self.label, label);
    }
}
