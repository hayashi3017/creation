pub mod diagram;
pub mod entity;
pub mod user;

macro_rules! map_service_result {
    ($future:expr, $error:path) => {{
        match $future.await {
            Err(err) => Err($error(err)),
            Ok(val) => Ok(val),
        }
    }};
}

macro_rules! map_service_result_unit {
    ($future:expr, $error:path) => {{
        match $future.await {
            Err(err) => Err($error(err)),
            Ok(_) => Ok(()),
        }
    }};
}

pub(crate) use map_service_result;
pub(crate) use map_service_result_unit;

#[cfg(test)]
mod tests {
    use super::{map_service_result, map_service_result_unit};

    #[derive(Debug, PartialEq)]
    enum WrappedError {
        Inner(u8),
    }

    #[test]
    fn map_service_result_maps_ok_and_err() {
        let ok = futures::executor::block_on(async {
            map_service_result!(async { Ok::<u16, u8>(42) }, WrappedError::Inner)
        });
        assert_eq!(ok, Ok(42));

        let err = futures::executor::block_on(async {
            map_service_result!(async { Err::<u16, u8>(7) }, WrappedError::Inner)
        });
        assert_eq!(err, Err(WrappedError::Inner(7)));
    }

    #[test]
    fn map_service_result_unit_maps_ok_and_err() {
        let ok = futures::executor::block_on(async {
            map_service_result_unit!(async { Ok::<usize, u8>(1) }, WrappedError::Inner)
        });
        assert_eq!(ok, Ok(()));

        let err = futures::executor::block_on(async {
            map_service_result_unit!(async { Err::<usize, u8>(3) }, WrappedError::Inner)
        });
        assert_eq!(err, Err(WrappedError::Inner(3)));
    }
}
