use std::path::{Path, PathBuf};

/// A context that is shared between multiple [`WebView`]s.
///
/// A browser would have a context for all the normal tabs and a different context for all the
/// private/incognito tabs.
pub struct WebContext {
  data: WebContextData,
  os: WebContextImpl,
}

impl WebContext {
  /// Create a new [`WebContext`].
  ///
  /// `data_directory`:
  /// * Whether the WebView window should have a custom user data path. This is useful in Windows
  ///   when a bundled application can't have the webview data inside `Program Files`.
  pub fn new(data_directory: Option<PathBuf>) -> Self {
    let data = WebContextData { data_directory };
    let os = WebContextImpl::new(&data);
    Self { data, os }
  }

  /// A reference to the data directory the context was created with.
  pub fn data_directory(&self) -> Option<&Path> {
    self.data.data_directory()
  }
}

impl Default for WebContext {
  fn default() -> Self {
    let data = WebContextData::default();
    let os = WebContextImpl::new(&data);
    Self { data, os }
  }
}

/// Data that all [`WebContext`] share regardless of platform.
#[derive(Default)]
struct WebContextData {
  data_directory: Option<PathBuf>,
}

impl WebContextData {
  /// A reference to the data directory the context was created with.
  pub fn data_directory(&self) -> Option<&Path> {
    self.data_directory.as_deref()
  }
}

#[cfg(not(target_os = "linux"))]
#[cfg_attr(doc_cfg, doc(cfg(not(target_os = "linux"))))]
pub struct WebContextImpl;

#[cfg(not(target_os = "linux"))]
#[cfg_attr(doc_cfg, doc(cfg(not(target_os = "linux"))))]
impl WebContextImpl {
  pub fn new(_data: &super::WebContextData) -> Self {
    Self
  }
}

#[cfg(target_os = "linux")]
use self::unix::WebContextImpl;

#[cfg(target_os = "linux")]
#[cfg_attr(doc_cfg, doc(cfg(target_os = "linux")))]
pub mod unix {
  //! Unix platform extensions for [`WebContext`](super::WebContext).

  use webkit2gtk::{
    ApplicationInfo, WebContext, WebContextBuilder, WebContextExt as WebkitWebContextExt,
    WebsiteDataManagerBuilder,
  };

  pub(super) struct WebContextImpl {
    app_info: ApplicationInfo,
    context: WebContext,
    automation: bool,
  }

  impl WebContextImpl {
    pub fn new(data: &super::WebContextData) -> Self {
      let mut context_builder = WebContextBuilder::new();
      if let Some(data_directory) = data.data_directory() {
        let data_manager = WebsiteDataManagerBuilder::new()
          .local_storage_directory(
            &data_directory
              .join("localstorage")
              .to_string_lossy()
              .into_owned(),
          )
          .indexeddb_directory(
            &data_directory
              .join("databases")
              .join("indexeddb")
              .to_string_lossy()
              .into_owned(),
          )
          .build();
        context_builder = context_builder.website_data_manager(&data_manager);
      }

      let context = context_builder.build();

      // default to true since other platforms don't have a way to disable it (yet)
      let automation = true;
      context.set_automation_allowed(automation);

      // e.g. wry 0.9.4
      let app_info = ApplicationInfo::new();
      app_info.set_name(env!("CARGO_PKG_NAME"));
      app_info.set_version(
        env!("CARGO_PKG_VERSION_MAJOR")
          .parse()
          .expect("invalid wry version major"),
        env!("CARGO_PKG_VERSION_MINOR")
          .parse()
          .expect("invalid wry version minor"),
        env!("CARGO_PKG_VERSION_PATCH")
          .parse()
          .expect("invalid wry version patch"),
      );

      Self {
        app_info,
        context,
        automation,
      }
    }
  }

  /// [`WebContext`](super::WebContext) items that only matter on unix.
  pub trait WebContextExt {
    /// The application info shared between webviews.
    fn app_info(&self) -> &ApplicationInfo;

    /// The context of all webviews opened.
    fn context(&self) -> &WebContext;

    /// If the context allows automation.
    ///
    /// **Note:** `libwebkit2gtk` only allows 1 automation context at a time.
    fn allows_automation(&self) -> bool;

    /// Set if this context allows automation.
    ///
    /// **Note:** `libwebkit2gtk` only allows 1 automation context at a time.
    fn set_allows_automation(&mut self, flag: bool);
  }

  impl WebContextExt for super::WebContext {
    fn app_info(&self) -> &ApplicationInfo {
      &self.os.app_info
    }

    fn context(&self) -> &WebContext {
      &self.os.context
    }

    fn allows_automation(&self) -> bool {
      self.os.automation
    }

    fn set_allows_automation(&mut self, flag: bool) {
      self.os.automation = flag;
      self.os.context.set_automation_allowed(flag);
    }
  }
}
