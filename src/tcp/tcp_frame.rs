use super::tcp_stream::SlowTcpStream;
use std::io;

const MAX_FRAME_SIZE: usize = 1024 * 1024; // 1MB limit

/// Represents a TCP frame in the slow network stack
pub struct SlowTcpFrame;

impl SlowTcpFrame {
    /// Sends data over the link with length-prefix framing.
    ///
    /// The data is wrapped with length prefixes at both the start and end for validation.
    /// There is a 1MB size limit for any single transmission.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    ///
    /// # Returns
    /// The number of bytes sent (not including framing)
    ///
    /// # Errors
    /// Returns an error if the data is too large or if the transmission fails
    pub async fn send(data: &[u8], stream: &SlowTcpStream) -> io::Result<usize> {
        // Ensure the data is not too large
        if data.len() > MAX_FRAME_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Data size exceeds 1MB limit: {} bytes", data.len()),
            ));
        }

        let len = data.len() as u32;
        let len_bytes = len.to_be_bytes();

        // Send the length prefix
        stream.write(&len_bytes).await?;

        // Send the actual data
        let bytes_sent = stream.write(data).await?;

        // Send the length suffix (same as prefix for validation)
        stream.write(&len_bytes).await?;

        // Return the number of data bytes sent (not including the length prefix/suffix)
        Ok(bytes_sent)
    }

    /// Receives data from the link with length-prefix framing validation.
    ///
    /// Reads a length prefix, the actual data, and a length suffix,
    /// validating that the prefix and suffix match.
    ///
    /// # Arguments
    /// * `buffer` - Buffer to store the received data
    ///
    /// # Returns
    /// The number of bytes read into the buffer
    ///
    /// # Errors
    /// Returns an error if the buffer is too small, if reading fails,
    /// or if the length prefix and suffix don't match
    pub async fn receive(buffer: &mut [u8], stream: &SlowTcpStream) -> io::Result<usize> {
        // First read the length prefix (4 bytes for u32)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;

        // Convert bytes to u32 (from network byte order)
        let expected_len = u32::from_be_bytes(len_bytes);

        // Ensure the buffer is large enough
        if buffer.len() < expected_len as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "SlowTcpLink: buffer too small: {} bytes needed but only {} bytes available.",
                    expected_len,
                    buffer.len()
                ),
            ));
        }

        // Now read the actual data using read_exact to ensure we get all the expected bytes
        stream
            .read_exact(&mut buffer[..expected_len as usize])
            .await?;

        // Read and validate the length suffix
        let mut suffix_len_bytes = [0u8; 4];
        stream.read_exact(&mut suffix_len_bytes).await?;
        let suffix_len = u32::from_be_bytes(suffix_len_bytes);

        // Ensure the length prefix matches the length suffix
        if expected_len != suffix_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Length mismatch: prefix {} bytes but suffix indicates {} bytes",
                    expected_len, suffix_len
                ),
            ));
        }

        // Return the number of data bytes read
        Ok(expected_len as usize)
    }

    /// Returns the maximum allowed frame size for this link implementation.
    ///
    /// # Returns
    /// The maximum number of bytes that can be sent in a single frame
    pub fn max_frame_size() -> usize {
        MAX_FRAME_SIZE
    }
}
