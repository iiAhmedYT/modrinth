// Ads have been removed from this privacy-focused build. These helpers are
// kept as no-ops so existing call sites continue to work without changes.

export async function init_ads_window(_overrideShown = false) {}

export async function show_ads_window() {}

export async function hide_ads_window(_reset) {}

export async function get_ads_consent_required() {
	return false
}

export async function perform_ads_consent_action(_action) {}

export async function open_ads_consent_preferences() {}

export async function ads_consent_listener(_callback) {
	// Return a no-op unlisten function to match Tauri's listen() contract.
	return () => {}
}

export async function record_ads_click() {}

export async function open_ads_link(_path, _origin) {}
