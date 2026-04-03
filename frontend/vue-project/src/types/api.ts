// Request types

export type SignupRequest = {
  name: string
  email: string
  password: string
}

export type LoginRequest = {
  email: string
  password: string
}

// Response types

export type StandardUserView = {
  name: string
  email: string
  is_admin: boolean
}
