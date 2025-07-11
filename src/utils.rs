pub trait LogResult<T, E> {
    /// Logs error message as `'{err}'` format, only on Err results. Returns Result
    fn log_err(self) -> Self;
    /// Logs error message as `'{msg} - {err}'` format, only on Err results. Returns Result
    fn log_err_msg(self, msg: impl AsRef<str>) -> Self;
    /// Logs  ok message as `'{msg}'` format, only on Ok results. Returns Result
    fn log_ok_msg(self, msg: impl AsRef<str>) -> Self;
}

impl<T, E> LogResult<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn log_ok_msg(self, msg: impl AsRef<str>) -> Self {
        let msg = msg.as_ref();
        match &self {
            Ok(_) => log::info!("{msg}"),
            _ => {}
        }
        self
    }

    fn log_err_msg(self, msg: impl AsRef<str>) -> Self {
        match &self {
            Ok(_) => {}
            Err(err) => {
                let msg = msg.as_ref();
                log::error!("{msg} - {err}");
            }
        }

        self
    }

    fn log_err(self) -> Self {
        match &self {
            Ok(_) => {}
            Err(err) => {
                log::error!("{err}");
            }
        }
        self
    }
}
