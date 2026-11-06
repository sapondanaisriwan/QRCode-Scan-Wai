//! Window lifecycle is independent of the platform capture backend and IPC.
use super::CaptureError;

pub trait CaptureSurface {
    fn hide(&self) -> Result<(), CaptureError>;
    fn restore(&self) -> Result<(), CaptureError>;
}
struct Restore<'a, S: CaptureSurface> {
    surface: &'a S,
    armed: bool,
}
impl<S: CaptureSurface> Drop for Restore<'_, S> {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.surface.restore();
        }
    }
}
/// Restore even if hiding partially fails, capture fails, or the worker unwinds.
pub fn with_hidden_surface<S: CaptureSurface, T>(
    surface: &S,
    capture: impl FnOnce() -> Result<T, CaptureError>,
) -> Result<T, CaptureError> {
    let mut guard = Restore {
        surface,
        armed: true,
    };
    let result = surface.hide().and_then(|()| capture());
    let restored = surface.restore();
    if restored.is_ok() {
        guard.armed = false;
    }
    restored?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Surface {
        events: RefCell<Vec<&'static str>>,
        hide_fails: bool,
    }
    impl CaptureSurface for Surface {
        fn hide(&self) -> Result<(), CaptureError> {
            self.events.borrow_mut().push("hide");
            if self.hide_fails {
                Err(CaptureError::WindowFailed)
            } else {
                Ok(())
            }
        }
        fn restore(&self) -> Result<(), CaptureError> {
            self.events.borrow_mut().push("restore");
            Ok(())
        }
    }
    #[test]
    fn hides_before_capture_and_restores_after_success_or_failure() {
        for result in [Ok(7), Err(CaptureError::CaptureDenied)] {
            let surface = Surface {
                events: RefCell::new(vec![]),
                hide_fails: false,
            };
            assert_eq!(
                with_hidden_surface(&surface, || {
                    surface.events.borrow_mut().push("capture");
                    result
                }),
                result
            );
            assert_eq!(*surface.events.borrow(), ["hide", "capture", "restore"]);
        }
    }
    #[test]
    fn failed_hide_never_captures_and_still_restores() {
        let surface = Surface {
            events: RefCell::new(vec![]),
            hide_fails: true,
        };
        let result = with_hidden_surface(&surface, || -> Result<(), CaptureError> {
            panic!("must not capture")
        });
        assert_eq!(result, Err(CaptureError::WindowFailed));
        assert_eq!(*surface.events.borrow(), ["hide", "restore"]);
    }
    #[test]
    fn unwinding_worker_restores_window() {
        let surface = Surface {
            events: RefCell::new(vec![]),
            hide_fails: false,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_hidden_surface(&surface, || -> Result<(), CaptureError> {
                panic!("injected failure")
            })
        }));
        assert!(result.is_err());
        assert_eq!(*surface.events.borrow(), ["hide", "restore"]);
    }
}
