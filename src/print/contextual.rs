use contextual::{Contextual, WithContext};
use std::collections::HashSet;
use std::fmt;

use super::{Options, Size};

pub trait PrintWithContext<C> {
	fn contextual_fmt_with(
		&self,
		context: &C,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> fmt::Result;
}

impl<'a, T: PrintWithContext<C> + ?Sized, C> PrintWithContext<C> for &'a T {
	fn contextual_fmt_with(
		&self,
		context: &C,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> fmt::Result {
		T::contextual_fmt_with(*self, context, f, options, indent)
	}
}

impl<'c, T: PrintWithContext<C>, C> super::Print for Contextual<T, &'c C> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		self.0.contextual_fmt_with(self.1, f, options, indent)
	}
}

pub trait PrintWithSizeAndContext<C> {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result;
}

impl<'a, T: PrintWithSizeAndContext<C> + ?Sized, C> PrintWithSizeAndContext<C> for &'a T {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		T::contextual_fmt_with_size(*self, context, f, options, indent, sizes, index)
	}
}

impl<'c, T: PrintWithSizeAndContext<C>, C> super::PrintWithSize for Contextual<T, &'c C> {
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		self.0
			.contextual_fmt_with_size(self.1, f, options, indent, sizes, index)
	}
}

pub trait PrecomputeSizeWithContext<C> {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size;
}

impl<'a, T: PrecomputeSizeWithContext<C> + ?Sized, C> PrecomputeSizeWithContext<C> for &'a T {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		T::contextual_pre_compute_size(*self, context, options, sizes)
	}
}

impl<'c, T: PrecomputeSizeWithContext<C>, C> super::PrecomputeSize for Contextual<T, &'c C> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.0.contextual_pre_compute_size(self.1, options, sizes)
	}
}

impl<T: PrecomputeSizeWithContext<C>, M, C> PrecomputeSizeWithContext<C> for locspan::Meta<T, M> {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		self.0.contextual_pre_compute_size(context, options, sizes)
	}
}

impl<T: PrintWithSizeAndContext<C>, M, C> PrintWithSizeAndContext<C> for locspan::Meta<T, M> {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		self.0
			.contextual_fmt_with_size(context, f, options, indent, sizes, index)
	}
}

impl<T: PrintWithContext<C>, M, C> PrintWithContext<C> for locspan::Meta<T, M> {
	fn contextual_fmt_with(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		self.0.contextual_fmt_with(context, f, options, indent)
	}
}

impl<T: PrecomputeSizeWithContext<C>, C> PrecomputeSizeWithContext<C> for locspan::Stripped<T> {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		self.0.contextual_pre_compute_size(context, options, sizes)
	}
}

impl<T: PrintWithSizeAndContext<C>, C> PrintWithSizeAndContext<C> for locspan::Stripped<T> {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		self.0
			.contextual_fmt_with_size(context, f, options, indent, sizes, index)
	}
}

impl<T: PrintWithContext<C>, C> PrintWithContext<C> for locspan::Stripped<T> {
	fn contextual_fmt_with(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		self.0.contextual_fmt_with(context, f, options, indent)
	}
}

impl<T: PrecomputeSizeWithContext<C>, C> PrecomputeSizeWithContext<C> for [T] {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		super::pre_compute_array_size(self.iter().map(|i| i.with(context)), options, sizes)
	}
}

impl<T: PrintWithSizeAndContext<C>, C> PrintWithSizeAndContext<C> for [T] {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		super::print_array(
			self.iter().map(|i| i.with(context)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<T: PrecomputeSizeWithContext<C>, C> PrecomputeSizeWithContext<C> for HashSet<T> {
	fn contextual_pre_compute_size(
		&self,
		context: &C,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		super::pre_compute_array_size(self.iter().map(|i| i.with(context)), options, sizes)
	}
}

impl<T: PrintWithSizeAndContext<C>, C> PrintWithSizeAndContext<C> for HashSet<T> {
	fn contextual_fmt_with_size(
		&self,
		context: &C,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		super::print_array(
			self.iter().map(|i| i.with(context)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}
