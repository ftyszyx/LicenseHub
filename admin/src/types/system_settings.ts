export interface SiteSettings {
  storefront_title: string
  registration_enabled: boolean
  resource_storage_channel_id: number
  distribution: DistributionSettings
  license_signing: LicenseSigningInfo
  email?: EmailSettings | null
}

export interface EmailSettings {
  mode: 'log' | 'smtp'
  from: string
  smtp_host: string
  smtp_port: number
  smtp_username: string
  smtp_password_set: boolean
  smtp_tls_mode: 'starttls' | 'tls' | 'none'
}

export interface DistributionSettings {
  enabled: boolean
  referrer_binding_enabled: boolean
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
  registration_enabled: boolean
  resource_storage_channel_id: number
  distribution_enabled: boolean
  distribution_referrer_binding_enabled: boolean
  distribution_default_rate_bps: number
  distribution_attribution_days: number
  distribution_holding_days: number
  distribution_min_withdraw_cents: number
  email_service_mode: 'log' | 'smtp'
  email_from: string
  email_smtp_host: string
  email_smtp_port: number
  email_smtp_username: string
  email_smtp_password?: string
  email_smtp_tls_mode: 'starttls' | 'tls' | 'none'
}

export interface GenerateLicenseSigningKeyReq {
  rotate: boolean
}
