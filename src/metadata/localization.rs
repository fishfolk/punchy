use std::borrow::Borrow;

use bevy_fluent::{exts::fluent::content::Request, BundleAsset, Content, Localization};
use fluent::FluentArgs;
use unic_langid::LanguageIdentifier;

use super::*;

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TranslationsMeta {
    /// The locale setting detected on the user's system
    #[serde(skip)]
    #[has_load_progress(none)]
    pub detected_locale: LanguageIdentifier,
    /// The default locale that will be used if a message is not found in the user's selected locale
    #[has_load_progress(none)]
    pub default_locale: LanguageIdentifier,
    /// Paths to the locale resources to load
    pub locales: Vec<String>,
    /// The handles to the locale bundle assets
    #[serde(skip)]
    pub locale_handles: Vec<Handle<BundleAsset>>,
}

/// Extension trait to reduce boilerplate when getting values from a [`Localization`].
pub trait LocalizationExt<'a, T: Into<Request<'a, U>>, U: Borrow<FluentArgs<'a>>> {
    /// Request message content and get an empty string if it doesn't exist.
    fn get(&self, request: T) -> String;
}

impl<'a, T, U> LocalizationExt<'a, T, U> for Localization
where
    T: Copy + Into<Request<'a, U>>,
    U: Borrow<FluentArgs<'a>>,
{
    /// Request message content and get an empty string if it doesn't exist.
    fn get(&self, request: T) -> String {
        self.content(request).unwrap_or_default()
    }
}
