export interface SiteSettings {
  storefront_title: string
  license_signing: LicenseSigningInfo
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
}

export interface GenerateLicenseSigningKeyReq {
  rotate: boolean
}
