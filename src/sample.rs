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

    pub(crate) fn map_sample<VV, F: FnOnce(V) -> VV>(self, f: F) -> Sample<VV> {
        Sample {
            label: self.label,
            sample: f(self.sample),
            valid: self.valid,
        }
    }

    pub(crate) fn add_label(&mut self, label: String) {
        self.label = format!("{}-{}", self.label, label);
    }
}
