export interface SiteSettings {
  storefront_title: string
  distribution: DistributionSettings
  license_signing: LicenseSigningInfo
}

export interface DistributionSettings {
  enabled: boolean
  default_rate_bps: number
  attribution_days: number
  holding_days: number
  min_withdraw_cents: number
}

export interface LicenseSigningInfo {
  configured: boolean
  key_id: string
  public_key_b64?: string | null
  private_key_b64?: string | null
  updated_at?: string | null
}

export interface SaveSystemSettingsReq {
  storefront_title: string
  distribution_enabled: boolean
  distribution_default_rate_bps: number
  distribution_attribution_days: number
  distribution_holding_days: number
  distribution_min_withdraw_cents: number
}

export interface GenerateLicenseSigningKeyReq {
  rotate: boolean
}
