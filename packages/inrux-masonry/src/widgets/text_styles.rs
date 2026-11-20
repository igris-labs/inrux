//! Text style types for the current shared Parley provider.
use masonry::core::BrushIndex;
pub(crate) type StyleProperty = parley::StyleProperty<'static, BrushIndex>;
pub(crate) type StyleSet = parley::StyleSet<BrushIndex>;
/// Match native default leading and family before application styles override them.
pub(crate) fn default_text_styles(styles: &mut StyleSet) {
    styles.insert(StyleProperty::LineHeight(
        parley::LineHeight::FontSizeRelative(1.2),
    ));
    styles.insert(parley::FontFamilyName::Generic(parley::GenericFamily::SystemUi).into());
}
