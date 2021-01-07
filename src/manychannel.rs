use rodio::Source;

// assuming each synth is only 1 channel
struct ManyChannel<T> {
    synths: Vec<T>,
    current_channel: usize,
}

impl<T: Source> ManyChannel<T>
where
    T::Item: rodio::Sample,
{
    fn new(synths: Vec<T>) -> Self {
        Self {
            synths,
            current_channel: 0,
        }
    }
}

impl<T: Source> Iterator for ManyChannel<T>
where
    T::Item: rodio::Sample,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.synths[self.current_channel].next();
        self.current_channel = (self.current_channel + 1) % self.synths.len();
        result
    }
}

impl<T: Source> Source for ManyChannel<T>
where
    <T as Iterator>::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.synths[self.current_channel].current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.synths.len() as _
    }

    fn sample_rate(&self) -> u32 {
        self.synths[self.current_channel].sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.synths[self.current_channel].total_duration()
    }
}
