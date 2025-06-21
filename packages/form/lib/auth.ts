import { formOptions } from '@tanstack/react-form'
import { z } from 'zod'

export const LOGIN_FORM = 'login'

export const loginFormOptions = () =>
  formOptions({
    defaultValues: {
      email: '',
      password: '',
    },
    validators: {
      onBlur: z.object({
        email: z.string().email('Email is not valid'),
        password: z.string().min(8, 'Password must be at least 8 characters'),
      }),
    },
  })

export const REGISTER_FORM = 'register'

export const registerFormOptions = () =>
  formOptions({
    defaultValues: {
      email: '',
      password: '',
      passwordConfirm: '',
    },
    validators: {
      onBlur: z.object({
        email: z.string().email('Email is not valid'),
        password: z.string().min(8, 'Password must be at least 8 characters'),
        passwordConfirm: z.string(),
      }),
    },
  })
