pub use root::*;

const _: () = ::planus::check_version_compatibility("planus-1.2.0");

/// The root namespace
///
/// Generated from these locations:
/// * File `cold_search_core/src/schema.fbs`
#[no_implicit_prelude]
#[allow(dead_code, clippy::needless_lifetimes)]
mod root {
    /// The table `TextWrapper`
    ///
    /// Generated from these locations:
    /// * Table `TextWrapper` in the file `cold_search_core/src/schema.fbs:1`
    #[derive(
        Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, ::serde::Serialize, ::serde::Deserialize,
    )]
    pub struct TextWrapper {
        /// The field `value` in the table `TextWrapper`
        pub value: ::core::option::Option<::planus::alloc::string::String>,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for TextWrapper {
        fn default() -> Self {
            Self {
                value: ::core::default::Default::default(),
            }
        }
    }

    impl TextWrapper {
        /// Creates a [TextWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> TextWrapperBuilder<()> {
            TextWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<::planus::Offset<str>>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 4>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<TextWrapper>> for TextWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TextWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<TextWrapper>> for TextWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<TextWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<TextWrapper> for TextWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TextWrapper> {
            TextWrapper::create(builder, &self.value)
        }
    }

    /// Builder for serializing an instance of the [TextWrapper] type.
    ///
    /// Can be created using the [TextWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct TextWrapperBuilder<State>(State);

    impl TextWrapperBuilder<()> {
        /// Setter for the [`value` field](TextWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> TextWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
        {
            TextWrapperBuilder((value,))
        }

        /// Sets the [`value` field](TextWrapper#structfield.value) to null.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_null(self) -> TextWrapperBuilder<((),)> {
            self.value(())
        }
    }

    impl<T0> TextWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [TextWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<TextWrapper>
        where
            Self: ::planus::WriteAsOffset<TextWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>>
        ::planus::WriteAs<::planus::Offset<TextWrapper>> for TextWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<TextWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TextWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>>
        ::planus::WriteAsOptional<::planus::Offset<TextWrapper>> for TextWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<TextWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<TextWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>>
        ::planus::WriteAsOffset<TextWrapper> for TextWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TextWrapper> {
            let (v0,) = &self.0;
            TextWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [TextWrapper].
    #[derive(Copy, Clone)]
    pub struct TextWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> TextWrapperRef<'a> {
        /// Getter for the [`value` field](TextWrapper#structfield.value).
        #[inline]
        pub fn value(
            &self,
        ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
            self.0.access(0, "TextWrapper", "value")
        }
    }

    impl<'a> ::core::fmt::Debug for TextWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("TextWrapperRef");
            if let ::core::option::Option::Some(field_value) = self.value().transpose() {
                f.field("value", &field_value);
            }
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<TextWrapperRef<'a>> for TextWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: TextWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: value.value()?.map(::core::convert::Into::into),
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for TextWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for TextWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location("[TextWrapperRef]", "get", buffer.offset_from_start)
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<TextWrapper>> for TextWrapper {
        type Value = ::planus::Offset<TextWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<TextWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for TextWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[TextWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The table `IntegerWrapper`
    ///
    /// Generated from these locations:
    /// * Table `IntegerWrapper` in the file `cold_search_core/src/schema.fbs:2`
    #[derive(
        Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, ::serde::Serialize, ::serde::Deserialize,
    )]
    pub struct IntegerWrapper {
        /// The field `value` in the table `IntegerWrapper`
        pub value: i64,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for IntegerWrapper {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl IntegerWrapper {
        /// Creates a [IntegerWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> IntegerWrapperBuilder<()> {
            IntegerWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsDefault<i64, i64>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder, &0);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<i64>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 8>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<IntegerWrapper>> for IntegerWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<IntegerWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<IntegerWrapper>> for IntegerWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<IntegerWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<IntegerWrapper> for IntegerWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<IntegerWrapper> {
            IntegerWrapper::create(builder, self.value)
        }
    }

    /// Builder for serializing an instance of the [IntegerWrapper] type.
    ///
    /// Can be created using the [IntegerWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct IntegerWrapperBuilder<State>(State);

    impl IntegerWrapperBuilder<()> {
        /// Setter for the [`value` field](IntegerWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> IntegerWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsDefault<i64, i64>,
        {
            IntegerWrapperBuilder((value,))
        }

        /// Sets the [`value` field](IntegerWrapper#structfield.value) to the default value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_default(self) -> IntegerWrapperBuilder<(::planus::DefaultValue,)> {
            self.value(::planus::DefaultValue)
        }
    }

    impl<T0> IntegerWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [IntegerWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<IntegerWrapper>
        where
            Self: ::planus::WriteAsOffset<IntegerWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>> ::planus::WriteAs<::planus::Offset<IntegerWrapper>>
        for IntegerWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<IntegerWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<IntegerWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>>
        ::planus::WriteAsOptional<::planus::Offset<IntegerWrapper>>
        for IntegerWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<IntegerWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<IntegerWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>> ::planus::WriteAsOffset<IntegerWrapper>
        for IntegerWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<IntegerWrapper> {
            let (v0,) = &self.0;
            IntegerWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [IntegerWrapper].
    #[derive(Copy, Clone)]
    pub struct IntegerWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> IntegerWrapperRef<'a> {
        /// Getter for the [`value` field](IntegerWrapper#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<i64> {
            ::core::result::Result::Ok(self.0.access(0, "IntegerWrapper", "value")?.unwrap_or(0))
        }
    }

    impl<'a> ::core::fmt::Debug for IntegerWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("IntegerWrapperRef");
            f.field("value", &self.value());
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<IntegerWrapperRef<'a>> for IntegerWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: IntegerWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: ::core::convert::TryInto::try_into(value.value()?)?,
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for IntegerWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for IntegerWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location(
                    "[IntegerWrapperRef]",
                    "get",
                    buffer.offset_from_start,
                )
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<IntegerWrapper>> for IntegerWrapper {
        type Value = ::planus::Offset<IntegerWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<IntegerWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for IntegerWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[IntegerWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The table `FloatWrapper`
    ///
    /// Generated from these locations:
    /// * Table `FloatWrapper` in the file `cold_search_core/src/schema.fbs:3`
    #[derive(Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize)]
    pub struct FloatWrapper {
        /// The field `value` in the table `FloatWrapper`
        pub value: f64,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for FloatWrapper {
        fn default() -> Self {
            Self { value: 0.0 }
        }
    }

    impl FloatWrapper {
        /// Creates a [FloatWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> FloatWrapperBuilder<()> {
            FloatWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsDefault<f64, f64>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder, &0.0);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<f64>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 8>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<FloatWrapper>> for FloatWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<FloatWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<FloatWrapper>> for FloatWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<FloatWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<FloatWrapper> for FloatWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<FloatWrapper> {
            FloatWrapper::create(builder, self.value)
        }
    }

    /// Builder for serializing an instance of the [FloatWrapper] type.
    ///
    /// Can be created using the [FloatWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct FloatWrapperBuilder<State>(State);

    impl FloatWrapperBuilder<()> {
        /// Setter for the [`value` field](FloatWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> FloatWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsDefault<f64, f64>,
        {
            FloatWrapperBuilder((value,))
        }

        /// Sets the [`value` field](FloatWrapper#structfield.value) to the default value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_default(self) -> FloatWrapperBuilder<(::planus::DefaultValue,)> {
            self.value(::planus::DefaultValue)
        }
    }

    impl<T0> FloatWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [FloatWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<FloatWrapper>
        where
            Self: ::planus::WriteAsOffset<FloatWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<f64, f64>> ::planus::WriteAs<::planus::Offset<FloatWrapper>>
        for FloatWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<FloatWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<FloatWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<f64, f64>>
        ::planus::WriteAsOptional<::planus::Offset<FloatWrapper>> for FloatWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<FloatWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<FloatWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsDefault<f64, f64>> ::planus::WriteAsOffset<FloatWrapper>
        for FloatWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<FloatWrapper> {
            let (v0,) = &self.0;
            FloatWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [FloatWrapper].
    #[derive(Copy, Clone)]
    pub struct FloatWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> FloatWrapperRef<'a> {
        /// Getter for the [`value` field](FloatWrapper#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<f64> {
            ::core::result::Result::Ok(self.0.access(0, "FloatWrapper", "value")?.unwrap_or(0.0))
        }
    }

    impl<'a> ::core::fmt::Debug for FloatWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("FloatWrapperRef");
            f.field("value", &self.value());
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<FloatWrapperRef<'a>> for FloatWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: FloatWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: ::core::convert::TryInto::try_into(value.value()?)?,
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for FloatWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for FloatWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location("[FloatWrapperRef]", "get", buffer.offset_from_start)
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<FloatWrapper>> for FloatWrapper {
        type Value = ::planus::Offset<FloatWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<FloatWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for FloatWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[FloatWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The table `BooleanWrapper`
    ///
    /// Generated from these locations:
    /// * Table `BooleanWrapper` in the file `cold_search_core/src/schema.fbs:4`
    #[derive(
        Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, ::serde::Serialize, ::serde::Deserialize,
    )]
    pub struct BooleanWrapper {
        /// The field `value` in the table `BooleanWrapper`
        pub value: bool,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for BooleanWrapper {
        fn default() -> Self {
            Self { value: false }
        }
    }

    impl BooleanWrapper {
        /// Creates a [BooleanWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> BooleanWrapperBuilder<()> {
            BooleanWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsDefault<bool, bool>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder, &false);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<bool>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 1>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<BooleanWrapper>> for BooleanWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<BooleanWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<BooleanWrapper>> for BooleanWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<BooleanWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<BooleanWrapper> for BooleanWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<BooleanWrapper> {
            BooleanWrapper::create(builder, self.value)
        }
    }

    /// Builder for serializing an instance of the [BooleanWrapper] type.
    ///
    /// Can be created using the [BooleanWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct BooleanWrapperBuilder<State>(State);

    impl BooleanWrapperBuilder<()> {
        /// Setter for the [`value` field](BooleanWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> BooleanWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsDefault<bool, bool>,
        {
            BooleanWrapperBuilder((value,))
        }

        /// Sets the [`value` field](BooleanWrapper#structfield.value) to the default value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_default(self) -> BooleanWrapperBuilder<(::planus::DefaultValue,)> {
            self.value(::planus::DefaultValue)
        }
    }

    impl<T0> BooleanWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [BooleanWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<BooleanWrapper>
        where
            Self: ::planus::WriteAsOffset<BooleanWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<bool, bool>>
        ::planus::WriteAs<::planus::Offset<BooleanWrapper>> for BooleanWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<BooleanWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<BooleanWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<bool, bool>>
        ::planus::WriteAsOptional<::planus::Offset<BooleanWrapper>>
        for BooleanWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<BooleanWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<BooleanWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsDefault<bool, bool>> ::planus::WriteAsOffset<BooleanWrapper>
        for BooleanWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<BooleanWrapper> {
            let (v0,) = &self.0;
            BooleanWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [BooleanWrapper].
    #[derive(Copy, Clone)]
    pub struct BooleanWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> BooleanWrapperRef<'a> {
        /// Getter for the [`value` field](BooleanWrapper#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<bool> {
            ::core::result::Result::Ok(
                self.0
                    .access(0, "BooleanWrapper", "value")?
                    .unwrap_or(false),
            )
        }
    }

    impl<'a> ::core::fmt::Debug for BooleanWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("BooleanWrapperRef");
            f.field("value", &self.value());
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<BooleanWrapperRef<'a>> for BooleanWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: BooleanWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: ::core::convert::TryInto::try_into(value.value()?)?,
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for BooleanWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for BooleanWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location(
                    "[BooleanWrapperRef]",
                    "get",
                    buffer.offset_from_start,
                )
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<BooleanWrapper>> for BooleanWrapper {
        type Value = ::planus::Offset<BooleanWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<BooleanWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for BooleanWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[BooleanWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The table `DateWrapper`
    ///
    /// Generated from these locations:
    /// * Table `DateWrapper` in the file `cold_search_core/src/schema.fbs:5`
    #[derive(
        Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, ::serde::Serialize, ::serde::Deserialize,
    )]
    pub struct DateWrapper {
        /// The field `value` in the table `DateWrapper`
        pub value: i32,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for DateWrapper {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl DateWrapper {
        /// Creates a [DateWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> DateWrapperBuilder<()> {
            DateWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsDefault<i32, i32>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder, &0);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<i32>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 4>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<DateWrapper>> for DateWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<DateWrapper>> for DateWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<DateWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<DateWrapper> for DateWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateWrapper> {
            DateWrapper::create(builder, self.value)
        }
    }

    /// Builder for serializing an instance of the [DateWrapper] type.
    ///
    /// Can be created using the [DateWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct DateWrapperBuilder<State>(State);

    impl DateWrapperBuilder<()> {
        /// Setter for the [`value` field](DateWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> DateWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsDefault<i32, i32>,
        {
            DateWrapperBuilder((value,))
        }

        /// Sets the [`value` field](DateWrapper#structfield.value) to the default value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_default(self) -> DateWrapperBuilder<(::planus::DefaultValue,)> {
            self.value(::planus::DefaultValue)
        }
    }

    impl<T0> DateWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [DateWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateWrapper>
        where
            Self: ::planus::WriteAsOffset<DateWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i32, i32>> ::planus::WriteAs<::planus::Offset<DateWrapper>>
        for DateWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<DateWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i32, i32>>
        ::planus::WriteAsOptional<::planus::Offset<DateWrapper>> for DateWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<DateWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<DateWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsDefault<i32, i32>> ::planus::WriteAsOffset<DateWrapper>
        for DateWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateWrapper> {
            let (v0,) = &self.0;
            DateWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [DateWrapper].
    #[derive(Copy, Clone)]
    pub struct DateWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> DateWrapperRef<'a> {
        /// Getter for the [`value` field](DateWrapper#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<i32> {
            ::core::result::Result::Ok(self.0.access(0, "DateWrapper", "value")?.unwrap_or(0))
        }
    }

    impl<'a> ::core::fmt::Debug for DateWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("DateWrapperRef");
            f.field("value", &self.value());
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<DateWrapperRef<'a>> for DateWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: DateWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: ::core::convert::TryInto::try_into(value.value()?)?,
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for DateWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for DateWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location("[DateWrapperRef]", "get", buffer.offset_from_start)
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<DateWrapper>> for DateWrapper {
        type Value = ::planus::Offset<DateWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<DateWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for DateWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[DateWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The table `DateTimeWrapper`
    ///
    /// Generated from these locations:
    /// * Table `DateTimeWrapper` in the file `cold_search_core/src/schema.fbs:6`
    #[derive(
        Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, ::serde::Serialize, ::serde::Deserialize,
    )]
    pub struct DateTimeWrapper {
        /// The field `value` in the table `DateTimeWrapper`
        pub value: i64,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for DateTimeWrapper {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl DateTimeWrapper {
        /// Creates a [DateTimeWrapperBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> DateTimeWrapperBuilder<()> {
            DateTimeWrapperBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_value: impl ::planus::WriteAsDefault<i64, i64>,
        ) -> ::planus::Offset<Self> {
            let prepared_value = field_value.prepare(builder, &0);

            let mut table_writer: ::planus::table_writer::TableWriter<6> =
                ::core::default::Default::default();
            if prepared_value.is_some() {
                table_writer.write_entry::<i64>(0);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 8>(&prepared_value);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<DateTimeWrapper>> for DateTimeWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateTimeWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<DateTimeWrapper>> for DateTimeWrapper {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<DateTimeWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<DateTimeWrapper> for DateTimeWrapper {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateTimeWrapper> {
            DateTimeWrapper::create(builder, self.value)
        }
    }

    /// Builder for serializing an instance of the [DateTimeWrapper] type.
    ///
    /// Can be created using the [DateTimeWrapper::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct DateTimeWrapperBuilder<State>(State);

    impl DateTimeWrapperBuilder<()> {
        /// Setter for the [`value` field](DateTimeWrapper#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T0>(self, value: T0) -> DateTimeWrapperBuilder<(T0,)>
        where
            T0: ::planus::WriteAsDefault<i64, i64>,
        {
            DateTimeWrapperBuilder((value,))
        }

        /// Sets the [`value` field](DateTimeWrapper#structfield.value) to the default value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_default(self) -> DateTimeWrapperBuilder<(::planus::DefaultValue,)> {
            self.value(::planus::DefaultValue)
        }
    }

    impl<T0> DateTimeWrapperBuilder<(T0,)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [DateTimeWrapper].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateTimeWrapper>
        where
            Self: ::planus::WriteAsOffset<DateTimeWrapper>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>>
        ::planus::WriteAs<::planus::Offset<DateTimeWrapper>> for DateTimeWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<DateTimeWrapper>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateTimeWrapper> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>>
        ::planus::WriteAsOptional<::planus::Offset<DateTimeWrapper>>
        for DateTimeWrapperBuilder<(T0,)>
    {
        type Prepared = ::planus::Offset<DateTimeWrapper>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<DateTimeWrapper>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<T0: ::planus::WriteAsDefault<i64, i64>> ::planus::WriteAsOffset<DateTimeWrapper>
        for DateTimeWrapperBuilder<(T0,)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DateTimeWrapper> {
            let (v0,) = &self.0;
            DateTimeWrapper::create(builder, v0)
        }
    }

    /// Reference to a deserialized [DateTimeWrapper].
    #[derive(Copy, Clone)]
    pub struct DateTimeWrapperRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> DateTimeWrapperRef<'a> {
        /// Getter for the [`value` field](DateTimeWrapper#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<i64> {
            ::core::result::Result::Ok(self.0.access(0, "DateTimeWrapper", "value")?.unwrap_or(0))
        }
    }

    impl<'a> ::core::fmt::Debug for DateTimeWrapperRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("DateTimeWrapperRef");
            f.field("value", &self.value());
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<DateTimeWrapperRef<'a>> for DateTimeWrapper {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: DateTimeWrapperRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                value: ::core::convert::TryInto::try_into(value.value()?)?,
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for DateTimeWrapperRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for DateTimeWrapperRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location(
                    "[DateTimeWrapperRef]",
                    "get",
                    buffer.offset_from_start,
                )
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<DateTimeWrapper>> for DateTimeWrapper {
        type Value = ::planus::Offset<DateTimeWrapper>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<DateTimeWrapper>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for DateTimeWrapperRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| {
                error_kind.with_error_location("[DateTimeWrapperRef]", "read_as_root", 0)
            })
        }
    }

    /// The union `Value`
    ///
    /// Generated from these locations:
    /// * Union `Value` in the file `cold_search_core/src/schema.fbs:8`
    #[derive(Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize)]
    pub enum Value {
        /// The variant `Text` in the union `Value`
        Text(::planus::alloc::boxed::Box<self::TextWrapper>),

        /// The variant `Integer` in the union `Value`
        Integer(::planus::alloc::boxed::Box<self::IntegerWrapper>),

        /// The variant `Float` in the union `Value`
        Float(::planus::alloc::boxed::Box<self::FloatWrapper>),

        /// The variant `Boolean` in the union `Value`
        Boolean(::planus::alloc::boxed::Box<self::BooleanWrapper>),

        /// The variant `Date` in the union `Value`
        Date(::planus::alloc::boxed::Box<self::DateWrapper>),

        /// The variant `DateTime` in the union `Value`
        DateTime(::planus::alloc::boxed::Box<self::DateTimeWrapper>),
    }

    impl Value {
        /// Creates a [ValueBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> ValueBuilder<::planus::Uninitialized> {
            ValueBuilder(::planus::Uninitialized)
        }

        #[inline]
        pub fn create_text(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::TextWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(1, value.prepare(builder).downcast())
        }

        #[inline]
        pub fn create_integer(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::IntegerWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(2, value.prepare(builder).downcast())
        }

        #[inline]
        pub fn create_float(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::FloatWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(3, value.prepare(builder).downcast())
        }

        #[inline]
        pub fn create_boolean(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::BooleanWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(4, value.prepare(builder).downcast())
        }

        #[inline]
        pub fn create_date(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::DateWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(5, value.prepare(builder).downcast())
        }

        #[inline]
        pub fn create_date_time(
            builder: &mut ::planus::Builder,
            value: impl ::planus::WriteAsOffset<self::DateTimeWrapper>,
        ) -> ::planus::UnionOffset<Self> {
            ::planus::UnionOffset::new(6, value.prepare(builder).downcast())
        }
    }

    impl ::planus::WriteAsUnion<Value> for Value {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Self> {
            match self {
                Self::Text(value) => Self::create_text(builder, value),
                Self::Integer(value) => Self::create_integer(builder, value),
                Self::Float(value) => Self::create_float(builder, value),
                Self::Boolean(value) => Self::create_boolean(builder, value),
                Self::Date(value) => Self::create_date(builder, value),
                Self::DateTime(value) => Self::create_date_time(builder, value),
            }
        }
    }

    impl ::planus::WriteAsOptionalUnion<Value> for Value {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Self>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }

    /// Builder for serializing an instance of the [Value] type.
    ///
    /// Can be created using the [Value::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct ValueBuilder<T>(T);

    impl ValueBuilder<::planus::Uninitialized> {
        /// Creates an instance of the [`Text` variant](Value#variant.Text).
        #[inline]
        pub fn text<T>(self, value: T) -> ValueBuilder<::planus::Initialized<1, T>>
        where
            T: ::planus::WriteAsOffset<self::TextWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }

        /// Creates an instance of the [`Integer` variant](Value#variant.Integer).
        #[inline]
        pub fn integer<T>(self, value: T) -> ValueBuilder<::planus::Initialized<2, T>>
        where
            T: ::planus::WriteAsOffset<self::IntegerWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }

        /// Creates an instance of the [`Float` variant](Value#variant.Float).
        #[inline]
        pub fn float<T>(self, value: T) -> ValueBuilder<::planus::Initialized<3, T>>
        where
            T: ::planus::WriteAsOffset<self::FloatWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }

        /// Creates an instance of the [`Boolean` variant](Value#variant.Boolean).
        #[inline]
        pub fn boolean<T>(self, value: T) -> ValueBuilder<::planus::Initialized<4, T>>
        where
            T: ::planus::WriteAsOffset<self::BooleanWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }

        /// Creates an instance of the [`Date` variant](Value#variant.Date).
        #[inline]
        pub fn date<T>(self, value: T) -> ValueBuilder<::planus::Initialized<5, T>>
        where
            T: ::planus::WriteAsOffset<self::DateWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }

        /// Creates an instance of the [`DateTime` variant](Value#variant.DateTime).
        #[inline]
        pub fn date_time<T>(self, value: T) -> ValueBuilder<::planus::Initialized<6, T>>
        where
            T: ::planus::WriteAsOffset<self::DateTimeWrapper>,
        {
            ValueBuilder(::planus::Initialized(value))
        }
    }

    impl<const N: u8, T> ValueBuilder<::planus::Initialized<N, T>> {
        /// Finish writing the builder to get an [UnionOffset](::planus::UnionOffset) to a serialized [Value].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value>
        where
            Self: ::planus::WriteAsUnion<Value>,
        {
            ::planus::WriteAsUnion::prepare(&self, builder)
        }
    }

    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<1, T>>
    where
        T: ::planus::WriteAsOffset<self::TextWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(1, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<1, T>>
    where
        T: ::planus::WriteAsOffset<self::TextWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }
    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<2, T>>
    where
        T: ::planus::WriteAsOffset<self::IntegerWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(2, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<2, T>>
    where
        T: ::planus::WriteAsOffset<self::IntegerWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }
    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<3, T>>
    where
        T: ::planus::WriteAsOffset<self::FloatWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(3, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<3, T>>
    where
        T: ::planus::WriteAsOffset<self::FloatWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }
    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<4, T>>
    where
        T: ::planus::WriteAsOffset<self::BooleanWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(4, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<4, T>>
    where
        T: ::planus::WriteAsOffset<self::BooleanWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }
    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<5, T>>
    where
        T: ::planus::WriteAsOffset<self::DateWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(5, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<5, T>>
    where
        T: ::planus::WriteAsOffset<self::DateWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }
    impl<T> ::planus::WriteAsUnion<Value> for ValueBuilder<::planus::Initialized<6, T>>
    where
        T: ::planus::WriteAsOffset<self::DateTimeWrapper>,
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Value> {
            ::planus::UnionOffset::new(6, (self.0).0.prepare(builder).downcast())
        }
    }

    impl<T> ::planus::WriteAsOptionalUnion<Value> for ValueBuilder<::planus::Initialized<6, T>>
    where
        T: ::planus::WriteAsOffset<self::DateTimeWrapper>,
    {
        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::UnionOffset<Value>> {
            ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
        }
    }

    /// Reference to a deserialized [Value].
    #[derive(Copy, Clone, Debug)]
    pub enum ValueRef<'a> {
        Text(self::TextWrapperRef<'a>),
        Integer(self::IntegerWrapperRef<'a>),
        Float(self::FloatWrapperRef<'a>),
        Boolean(self::BooleanWrapperRef<'a>),
        Date(self::DateWrapperRef<'a>),
        DateTime(self::DateTimeWrapperRef<'a>),
    }

    impl<'a> ::core::convert::TryFrom<ValueRef<'a>> for Value {
        type Error = ::planus::Error;

        fn try_from(value: ValueRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(match value {
                ValueRef::Text(value) => Self::Text(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),

                ValueRef::Integer(value) => Self::Integer(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),

                ValueRef::Float(value) => Self::Float(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),

                ValueRef::Boolean(value) => Self::Boolean(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),

                ValueRef::Date(value) => Self::Date(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),

                ValueRef::DateTime(value) => Self::DateTime(::planus::alloc::boxed::Box::new(
                    ::core::convert::TryFrom::try_from(value)?,
                )),
            })
        }
    }

    impl<'a> ::planus::TableReadUnion<'a> for ValueRef<'a> {
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            tag: u8,
            field_offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            match tag {
                1 => ::core::result::Result::Ok(Self::Text(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                2 => ::core::result::Result::Ok(Self::Integer(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                3 => ::core::result::Result::Ok(Self::Float(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                4 => ::core::result::Result::Ok(Self::Boolean(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                5 => ::core::result::Result::Ok(Self::Date(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                6 => ::core::result::Result::Ok(Self::DateTime(::planus::TableRead::from_buffer(
                    buffer,
                    field_offset,
                )?)),
                _ => ::core::result::Result::Err(::planus::errors::ErrorKind::UnknownUnionTag {
                    tag,
                }),
            }
        }
    }

    impl<'a> ::planus::VectorReadUnion<'a> for ValueRef<'a> {
        const VECTOR_NAME: &'static str = "[ValueRef]";
    }

    /// The table `Field`
    ///
    /// Generated from these locations:
    /// * Table `Field` in the file `cold_search_core/src/schema.fbs:17`
    #[derive(Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize)]
    pub struct Field {
        /// The field `key` in the table `Field`
        pub key: ::core::option::Option<::planus::alloc::string::String>,
        /// The field `value` in the table `Field`
        pub value: ::core::option::Option<self::Value>,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for Field {
        fn default() -> Self {
            Self {
                key: ::core::default::Default::default(),
                value: ::core::default::Default::default(),
            }
        }
    }

    impl Field {
        /// Creates a [FieldBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> FieldBuilder<()> {
            FieldBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_key: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            field_value: impl ::planus::WriteAsOptionalUnion<self::Value>,
        ) -> ::planus::Offset<Self> {
            let prepared_key = field_key.prepare(builder);
            let prepared_value = field_value.prepare(builder);

            let mut table_writer: ::planus::table_writer::TableWriter<10> =
                ::core::default::Default::default();
            if prepared_key.is_some() {
                table_writer.write_entry::<::planus::Offset<str>>(0);
            }
            if prepared_value.is_some() {
                table_writer.write_entry::<::planus::Offset<self::Value>>(2);
            }
            if prepared_value.is_some() {
                table_writer.write_entry::<u8>(1);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_key) = prepared_key {
                        object_writer.write::<_, _, 4>(&prepared_key);
                    }
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 4>(&prepared_value.offset());
                    }
                    if let ::core::option::Option::Some(prepared_value) = prepared_value {
                        object_writer.write::<_, _, 1>(&prepared_value.tag());
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<Field>> for Field {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Field> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<Field>> for Field {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<Field>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<Field> for Field {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Field> {
            Field::create(builder, &self.key, &self.value)
        }
    }

    /// Builder for serializing an instance of the [Field] type.
    ///
    /// Can be created using the [Field::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct FieldBuilder<State>(State);

    impl FieldBuilder<()> {
        /// Setter for the [`key` field](Field#structfield.key).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn key<T0>(self, value: T0) -> FieldBuilder<(T0,)>
        where
            T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
        {
            FieldBuilder((value,))
        }

        /// Sets the [`key` field](Field#structfield.key) to null.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn key_as_null(self) -> FieldBuilder<((),)> {
            self.key(())
        }
    }

    impl<T0> FieldBuilder<(T0,)> {
        /// Setter for the [`value` field](Field#structfield.value).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value<T1>(self, value: T1) -> FieldBuilder<(T0, T1)>
        where
            T1: ::planus::WriteAsOptionalUnion<self::Value>,
        {
            let (v0,) = self.0;
            FieldBuilder((v0, value))
        }

        /// Sets the [`value` field](Field#structfield.value) to null.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn value_as_null(self) -> FieldBuilder<(T0, ())> {
            self.value(())
        }
    }

    impl<T0, T1> FieldBuilder<(T0, T1)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Field].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Field>
        where
            Self: ::planus::WriteAsOffset<Field>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            T1: ::planus::WriteAsOptionalUnion<self::Value>,
        > ::planus::WriteAs<::planus::Offset<Field>> for FieldBuilder<(T0, T1)>
    {
        type Prepared = ::planus::Offset<Field>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Field> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            T1: ::planus::WriteAsOptionalUnion<self::Value>,
        > ::planus::WriteAsOptional<::planus::Offset<Field>> for FieldBuilder<(T0, T1)>
    {
        type Prepared = ::planus::Offset<Field>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<Field>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            T1: ::planus::WriteAsOptionalUnion<self::Value>,
        > ::planus::WriteAsOffset<Field> for FieldBuilder<(T0, T1)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Field> {
            let (v0, v1) = &self.0;
            Field::create(builder, v0, v1)
        }
    }

    /// Reference to a deserialized [Field].
    #[derive(Copy, Clone)]
    pub struct FieldRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> FieldRef<'a> {
        /// Getter for the [`key` field](Field#structfield.key).
        #[inline]
        pub fn key(&self) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
            self.0.access(0, "Field", "key")
        }

        /// Getter for the [`value` field](Field#structfield.value).
        #[inline]
        pub fn value(&self) -> ::planus::Result<::core::option::Option<self::ValueRef<'a>>> {
            self.0.access_union(1, "Field", "value")
        }
    }

    impl<'a> ::core::fmt::Debug for FieldRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("FieldRef");
            if let ::core::option::Option::Some(field_key) = self.key().transpose() {
                f.field("key", &field_key);
            }
            if let ::core::option::Option::Some(field_value) = self.value().transpose() {
                f.field("value", &field_value);
            }
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<FieldRef<'a>> for Field {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: FieldRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                key: value.key()?.map(::core::convert::Into::into),
                value: if let ::core::option::Option::Some(value) = value.value()? {
                    ::core::option::Option::Some(::core::convert::TryInto::try_into(value)?)
                } else {
                    ::core::option::Option::None
                },
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for FieldRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for FieldRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location("[FieldRef]", "get", buffer.offset_from_start)
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<Field>> for Field {
        type Value = ::planus::Offset<Field>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<Field>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for FieldRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| error_kind.with_error_location("[FieldRef]", "read_as_root", 0))
        }
    }

    /// The table `Entry`
    ///
    /// Generated from these locations:
    /// * Table `Entry` in the file `cold_search_core/src/schema.fbs:22`
    #[derive(Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize)]
    pub struct Entry {
        /// The field `fields` in the table `Entry`
        pub fields: ::core::option::Option<::planus::alloc::vec::Vec<self::Field>>,
        /// The field `geometry` in the table `Entry`
        pub geometry: ::core::option::Option<::planus::alloc::vec::Vec<u8>>,
    }

    #[allow(clippy::derivable_impls)]
    impl ::core::default::Default for Entry {
        fn default() -> Self {
            Self {
                fields: ::core::default::Default::default(),
                geometry: ::core::default::Default::default(),
            }
        }
    }

    impl Entry {
        /// Creates a [EntryBuilder] for serializing an instance of this table.
        #[inline]
        pub fn builder() -> EntryBuilder<()> {
            EntryBuilder(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            builder: &mut ::planus::Builder,
            field_fields: impl ::planus::WriteAsOptional<
                ::planus::Offset<[::planus::Offset<self::Field>]>,
            >,
            field_geometry: impl ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
        ) -> ::planus::Offset<Self> {
            let prepared_fields = field_fields.prepare(builder);
            let prepared_geometry = field_geometry.prepare(builder);

            let mut table_writer: ::planus::table_writer::TableWriter<8> =
                ::core::default::Default::default();
            if prepared_fields.is_some() {
                table_writer.write_entry::<::planus::Offset<[::planus::Offset<self::Field>]>>(0);
            }
            if prepared_geometry.is_some() {
                table_writer.write_entry::<::planus::Offset<[u8]>>(1);
            }

            unsafe {
                table_writer.finish(builder, |object_writer| {
                    if let ::core::option::Option::Some(prepared_fields) = prepared_fields {
                        object_writer.write::<_, _, 4>(&prepared_fields);
                    }
                    if let ::core::option::Option::Some(prepared_geometry) = prepared_geometry {
                        object_writer.write::<_, _, 4>(&prepared_geometry);
                    }
                });
            }
            builder.current_offset()
        }
    }

    impl ::planus::WriteAs<::planus::Offset<Entry>> for Entry {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Entry> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl ::planus::WriteAsOptional<::planus::Offset<Entry>> for Entry {
        type Prepared = ::planus::Offset<Self>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<Entry>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl ::planus::WriteAsOffset<Entry> for Entry {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Entry> {
            Entry::create(builder, &self.fields, &self.geometry)
        }
    }

    /// Builder for serializing an instance of the [Entry] type.
    ///
    /// Can be created using the [Entry::builder] method.
    #[derive(Debug)]
    #[must_use]
    pub struct EntryBuilder<State>(State);

    impl EntryBuilder<()> {
        /// Setter for the [`fields` field](Entry#structfield.fields).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn fields<T0>(self, value: T0) -> EntryBuilder<(T0,)>
        where
            T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Field>]>>,
        {
            EntryBuilder((value,))
        }

        /// Sets the [`fields` field](Entry#structfield.fields) to null.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn fields_as_null(self) -> EntryBuilder<((),)> {
            self.fields(())
        }
    }

    impl<T0> EntryBuilder<(T0,)> {
        /// Setter for the [`geometry` field](Entry#structfield.geometry).
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn geometry<T1>(self, value: T1) -> EntryBuilder<(T0, T1)>
        where
            T1: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
        {
            let (v0,) = self.0;
            EntryBuilder((v0, value))
        }

        /// Sets the [`geometry` field](Entry#structfield.geometry) to null.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn geometry_as_null(self) -> EntryBuilder<(T0, ())> {
            self.geometry(())
        }
    }

    impl<T0, T1> EntryBuilder<(T0, T1)> {
        /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Entry].
        #[inline]
        pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Entry>
        where
            Self: ::planus::WriteAsOffset<Entry>,
        {
            ::planus::WriteAsOffset::prepare(&self, builder)
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Field>]>>,
            T1: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
        > ::planus::WriteAs<::planus::Offset<Entry>> for EntryBuilder<(T0, T1)>
    {
        type Prepared = ::planus::Offset<Entry>;

        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Entry> {
            ::planus::WriteAsOffset::prepare(self, builder)
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Field>]>>,
            T1: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
        > ::planus::WriteAsOptional<::planus::Offset<Entry>> for EntryBuilder<(T0, T1)>
    {
        type Prepared = ::planus::Offset<Entry>;

        #[inline]
        fn prepare(
            &self,
            builder: &mut ::planus::Builder,
        ) -> ::core::option::Option<::planus::Offset<Entry>> {
            ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
        }
    }

    impl<
            T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Field>]>>,
            T1: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
        > ::planus::WriteAsOffset<Entry> for EntryBuilder<(T0, T1)>
    {
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Entry> {
            let (v0, v1) = &self.0;
            Entry::create(builder, v0, v1)
        }
    }

    /// Reference to a deserialized [Entry].
    #[derive(Copy, Clone)]
    pub struct EntryRef<'a>(::planus::table_reader::Table<'a>);

    impl<'a> EntryRef<'a> {
        /// Getter for the [`fields` field](Entry#structfield.fields).
        #[inline]
        pub fn fields(
            &self,
        ) -> ::planus::Result<
            ::core::option::Option<::planus::Vector<'a, ::planus::Result<self::FieldRef<'a>>>>,
        > {
            self.0.access(0, "Entry", "fields")
        }

        /// Getter for the [`geometry` field](Entry#structfield.geometry).
        #[inline]
        pub fn geometry(&self) -> ::planus::Result<::core::option::Option<&'a [u8]>> {
            self.0.access(1, "Entry", "geometry")
        }
    }

    impl<'a> ::core::fmt::Debug for EntryRef<'a> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let mut f = f.debug_struct("EntryRef");
            if let ::core::option::Option::Some(field_fields) = self.fields().transpose() {
                f.field("fields", &field_fields);
            }
            if let ::core::option::Option::Some(field_geometry) = self.geometry().transpose() {
                f.field("geometry", &field_geometry);
            }
            f.finish()
        }
    }

    impl<'a> ::core::convert::TryFrom<EntryRef<'a>> for Entry {
        type Error = ::planus::Error;

        #[allow(unreachable_code)]
        fn try_from(value: EntryRef<'a>) -> ::planus::Result<Self> {
            ::core::result::Result::Ok(Self {
                fields: if let ::core::option::Option::Some(fields) = value.fields()? {
                    ::core::option::Option::Some(fields.to_vec_result()?)
                } else {
                    ::core::option::Option::None
                },
                geometry: value.geometry()?.map(|v| v.to_vec()),
            })
        }
    }

    impl<'a> ::planus::TableRead<'a> for EntryRef<'a> {
        #[inline]
        fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
            ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                buffer, offset,
            )?))
        }
    }

    impl<'a> ::planus::VectorReadInner<'a> for EntryRef<'a> {
        type Error = ::planus::Error;
        const STRIDE: usize = 4;

        unsafe fn from_buffer(
            buffer: ::planus::SliceWithStartOffset<'a>,
            offset: usize,
        ) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                error_kind.with_error_location("[EntryRef]", "get", buffer.offset_from_start)
            })
        }
    }

    /// # Safety
    /// The planus compiler generates implementations that initialize
    /// the bytes in `write_values`.
    unsafe impl ::planus::VectorWrite<::planus::Offset<Entry>> for Entry {
        type Value = ::planus::Offset<Entry>;
        const STRIDE: usize = 4;
        #[inline]
        fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
            ::planus::WriteAs::prepare(self, builder)
        }

        #[inline]
        unsafe fn write_values(
            values: &[::planus::Offset<Entry>],
            bytes: *mut ::core::mem::MaybeUninit<u8>,
            buffer_position: u32,
        ) {
            let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
            for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                ::planus::WriteAsPrimitive::write(
                    v,
                    ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                    buffer_position - (Self::STRIDE * i) as u32,
                );
            }
        }
    }

    impl<'a> ::planus::ReadAsRoot<'a> for EntryRef<'a> {
        fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
            ::planus::TableRead::from_buffer(
                ::planus::SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .map_err(|error_kind| error_kind.with_error_location("[EntryRef]", "read_as_root", 0))
        }
    }
}
