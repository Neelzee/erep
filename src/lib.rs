#[derive(Debug, Clone)]
pub struct Erep<T> {
    val: T,
    rep: Option<Ereport>,
}

impl<T> Erep<T> {
    /// Maps a function F, on value T, returning U.
    ///
    /// Used if function F does not error, and if you want to keep the possible `Ereport`
    ///
    /// # Example
    /// ```rust
    /// let err: Erep<i32> = Erep { val: 2, rep: None };
    ///
    /// err.vmap(|i| i + 2);
    ///
    /// assert_eq!(err.val, 4);
    /// assert_eq!(err.rep, None)
    /// ```
    pub fn vmap<F, U>(self, f: F) -> Erep<U>
    where
        F: Fn(T) -> U,
    {
        Erep {
            val: f(self.val),
            rep: self.rep,
        }
    }

    /// Maps a function F, on value T, returning Erep<U>.
    ///
    /// Used if function F may return an error, and if you want to keep the possible `Ereport`
    /// and combine it with the current Erep
    ///
    /// # Example
    /// ```rust
    /// let err: Erep<i32> = Erep { val: 2, rep: None };
    ///
    /// err.map(|i| Erep { val: i + 2, rep: None });
    ///
    /// assert_eq!(err.val, 4);
    /// assert_eq!(err.rep, None)
    /// ```
    pub fn map<F, U>(self, f: F) -> Erep<U>
    where
        F: Fn(T) -> Erep<U>,
    {
        let (vo, ro) = self.unwrap_with_err();
        let (v, r) = f(vo).unwrap_with_err();
        Erep {
            val: v,
            rep: Some(
                vec![ro, r]
                    .into_iter()
                    .fold(Ereport::empty(), Ereport::push_opt),
            ),
        }
    }

    /// Maps a function F, on value T, returning Result<U, E>, turning it from
    /// Erep<T> to Erep<Option<U>>.
    ///
    /// Used if function F may return an error, and if you want to keep the possible `Ereport`
    /// and combine it with the current Erep
    ///
    /// # Example
    /// ```rust
    /// let err: Erep<&str> = Erep { val: "2", rep: None };
    ///
    /// err.emap(|i| i.parse::<i32>());
    ///
    /// assert_eq!(err.val, 2);
    /// assert_eq!(err.rep, None)
    /// ```
    pub fn emap<F, U, E>(self, f: F, msg: Option<String>) -> Erep<Option<U>>
    where
        F: Fn(T) -> Result<U, E>,
        E: std::fmt::Debug,
    {
        let (vo, ro) = self.unwrap_with_err();

        match (f(vo), msg) {
            (Ok(v), _) => Erep {
                val: Some(v),
                rep: ro,
            },
            (Err(_), Some(m)) => Erep {
                val: None::<U>,
                rep: Some(Ereport::new(m)),
            },
            (Err(e), None) => Erep {
                val: None::<U>,
                rep: Some(Ereport::new(format!("{e:?}"))),
            },
        }
    }

    /// Maps a function F, on value T, returning Option<U>, turning it from
    /// Erep<T> to Erep<Option<U>>.
    ///
    /// Used if function F may return an error, and if you want to keep the possible `Ereport`
    /// and combine it with the current Erep
    ///
    /// # Example
    /// ```rust
    /// let err: Erep<&str> = Erep { val: "2", rep: None };
    ///
    /// err.omap(|i| i.parse::<i32>().ok());
    ///
    /// assert_eq!(err.val, 2);
    /// assert_eq!(err.rep, None)
    /// ```
    pub fn omap<F, U>(self, f: F, msg: Option<String>) -> Erep<Option<U>>
    where
        F: Fn(T) -> Option<U>,
    {
        match (f(self.val), msg) {
            (Some(val), _) => Erep {
                val: Some(val),
                rep: self.rep,
            },
            (None, None) => Erep {
                val: None::<U>,
                rep: Some(Ereport::new("Got None, instead of Some")),
            },
            (None, Some(msg)) => Erep {
                val: None::<U>,
                rep: Some(Ereport::new(msg)),
            },
        }
    }

    pub fn unwrap_with_err(self) -> (T, Option<Ereport>) {
        (self.val, self.rep)
    }
}

#[derive(Debug, Clone)]
pub struct Ereport {
    msg: String,
    stack: Vec<Ereport>,
}

impl Ereport {
    pub fn empty() -> Self {
        Self {
            msg: String::new(),
            stack: Vec::new(),
        }
    }

    pub fn push(self, other: Ereport) -> Self {
        let mut stack = self.stack;
        stack.push(other);
        Self { stack, ..self }
    }

    pub fn push_opt(self, other: Option<Ereport>) -> Self {
        match other {
            Some(e) => self.push(e),
            None => self,
        }
    }

    pub fn new<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            msg: msg.into(),
            stack: Vec::new(),
        }
    }
}
