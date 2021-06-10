# Data model
- Screen
  - Frame buffer
  - Mouse position
  - Mouse button states
  - Cursor sprite
  - Per portal
    - Position
    - Pixel data handle

# Thread model
- Screen thread
  - Remains blocked on a semaphore
  - Unblocked when one of the other threads wants to update something
    - Updates `InputDb` with the cursor position
    - Updates `???Db` with portal positions and pixel data
    - Sends events to client event pipes
- Mouse thread
  - Remains blocked on the `ps2mouse` device
  - When the mouse moves:
    - Updates the cursor position
    - Redraws the cursor sprite on the frame buffer
    - Releases the semaphore to unblock the screen thread
- Client threads
  - Remains blocked on the corresponding client message pipe
  - When a client sends a message:
    - Updates a portal as appropriate
    - Releases the semaphore to unblock the screen thread
