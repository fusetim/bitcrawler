pub trait CompactPeerInfo : PartialEq + Eq + Clone {
    /// The type of the peer id.
    type Error;

    /// Reads a compact peer info from a string.
    ///
    /// # Parameters
    ///
    /// - `data`: The string containing the compact peer info.
    ///
    /// # Returns
    ///
    /// A tuple containing the number of bytes read and the compact peer info if successful.
    /// Otherwise, an error is returned.
    ///
    /// # Errors
    ///
    /// An error is returned if the string does not contain a valid compact peer info.
    fn try_read_compact_peer_info(data: &[u8]) -> Result<(usize, Self), Self::Error>;

    /// Produces a compact peer info from the given peer info.
    /// 
    /// # Returns
    /// 
    /// A string (CoW) containing the compact peer info.
    fn write_compact_peer_info(&self) -> Vec<u8>;
}