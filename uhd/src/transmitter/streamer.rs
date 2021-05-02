use std::marker::PhantomData;
use std::ptr;

use super::metadata::TransmitMetadata;
use crate::error::{check_status, Error};
use crate::stream::StreamCommand;
use crate::usrp::Usrp;
use std::os::raw::c_void;

/// A streamer used to transmit samples from a USRP
///
/// The type parameter I is the type of sample that this streamer transmits.
#[derive(Debug)]
pub struct TransmitStreamer<'usrp, I> {
    /// Streamer handle
    handle: uhd_sys::uhd_tx_streamer_handle,

    /// A vector of pointers to buffers (used in transmit() to convert `&mut [&mut [I]]` to `*mut *mut I`
    /// without reallocating memory each time
    ///
    /// Invariant: If this is not empty, its length is equal to the value returned by
    /// self.num_channels().
    buffer_pointers: Vec<*mut c_void>,
    /// Link to the USRP that this streamer is associated with
    usrp: PhantomData<&'usrp Usrp>,
    /// Item type phantom data
    item_phantom: PhantomData<I>,
}

impl<I> TransmitStreamer<'_, I> {
    /// Creates a transmit streamer with a null streamer handle (for internal use only)
    ///
    /// After creating a streamer with this function, its streamer handle must be initialized.
    pub(crate) fn new() -> Self {
        TransmitStreamer {
            handle: ptr::null_mut(),
            buffer_pointers: Vec::new(),
            usrp: PhantomData,
            item_phantom: PhantomData,
        }
    }

    /// Returns a reference to the streamer handle
    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_tx_streamer_handle {
        &mut self.handle
    }
    /// Returns the streamer handle
    pub(crate) fn handle(&mut self) -> uhd_sys::uhd_tx_streamer_handle {
        self.handle
    }

    /// Sends a stream command to the USRP
    ///
    /// This can be used to start or stop streaming
    pub fn send_command(&self, command: &StreamCommand) -> Result<(), Error> {
        todo!()
        // let command_c = command.as_c_command();
        // check_status(unsafe { uhd_sys::uhd_tx_streamer_issue_stream_cmd(self.handle, &command_c) })
    }

    /// Returns the number of channels that this streamer is associated with
    pub fn num_channels(&self) -> usize {
        let mut num_channels = 0usize;
        check_status(unsafe {
            uhd_sys::uhd_tx_streamer_num_channels(
                self.handle,
                &mut num_channels as *mut usize as *mut _,
            )
        })
        .unwrap();
        num_channels
    }

    /// transmits samples from the USRP
    ///
    /// buffers: One or more buffers (one per channel) where the samples will be written. All
    /// buffers should have the same length. This function will panic if the number of buffers is
    /// not equal to self.num_channels(), or if not all buffers have the same length.
    ///
    /// timeout: The timeout for the transmit operation, in seconds
    ///
    /// one_packet: If this is true, one call to transmit() will not copy samples from more than
    /// one packet of the underlying protocol
    ///
    /// On success, this function returns a transmitMetadata object with information about
    /// the number of samples actually transmitd.
    pub fn transmit(
        &mut self,
        buffers: &mut [&mut [I]],
        timeout: f64,
        one_packet: bool,
    ) -> Result<TransmitMetadata, Error> {
        let mut metadata = TransmitMetadata::default();
        let mut samples_transmitd = 0usize;

        // Initialize buffer_pointers
        if self.buffer_pointers.is_empty() {
            self.buffer_pointers
                .resize(self.num_channels(), ptr::null_mut());
        }
        // Now buffer_pointers.len() is equal to self.num_channels().
        assert_eq!(
            buffers.len(),
            self.buffer_pointers.len(),
            "Number of buffers is not equal to this streamer's number of channels"
        );
        // Check that all buffers have the same length
        let buffer_length = check_equal_buffer_lengths(buffers);

        // Copy buffer pointers into C-compatible form
        for (entry, buffer) in self.buffer_pointers.iter_mut().zip(buffers.iter_mut()) {
            *entry = buffer.as_mut_ptr() as *mut c_void;
        }

        // check_status(unsafe {
        //     uhd_sys::uhd_tx_streamer_send(
        //         self.handle,
        //         self.buffer_pointers.as_mut_ptr(),
        //         buffer_length as _,
        //         metadata.handle_mut(),
        //         timeout,
        //         one_packet,
        //         &mut samples_transmitd as *mut usize as *mut _,
        //     )
        // })?;
        // metadata.set_samples(samples_transmitd);

        Ok(metadata)
    }

    /// transmits samples on a single channel with a timeout of 0.1 seconds and one_packet disabled
    pub fn transmit_simple(&mut self, buffer: &mut [I]) -> Result<TransmitMetadata, Error> {
        self.transmit(&mut [buffer], 0.1, false)
    }
}

/// Checks that all provided buffers have the same length. Returns the length of the buffers,
/// or 0 if there are no buffers. Panics if the buffer lengths are not equal.
fn check_equal_buffer_lengths<I>(buffers: &mut [&mut [I]]) -> usize {
    buffers
        .iter()
        .fold(None, |prev_size, buffer| {
            match prev_size {
                None => {
                    // Store the size of the first buffer
                    Some(buffer.len())
                }
                Some(prev_size) => {
                    assert_eq!(prev_size, buffer.len(), "Unequal buffer sizes");
                    Some(prev_size)
                }
            }
        })
        .unwrap_or(0)
}

impl<I> Drop for TransmitStreamer<'_, I> {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_tx_streamer_free(&mut self.handle) };
    }
}

// Thread safety: see https://files.ettus.com/manual/page_general.html#general_threading
// All functions are thread-safe, except that the uhd_tx_streamer send(), uhd_tx_streamer recv(), and
// uhd_tx_streamer recv_async_msg() functions. The corresponding Rust wrapper functions take &mut
// self, which enforces single-thread access.
unsafe impl<I> Send for TransmitStreamer<'_, I> {}
unsafe impl<I> Sync for TransmitStreamer<'_, I> {}
