import { FormatDistanceToken, formatDistanceToNowStrict } from 'date-fns'
import * as locale from 'date-fns/locale/en-US'

const formatDistanceLocale: Record<FormatDistanceToken, string> = {
  lessThanXSeconds: 's',
  xSeconds: 's',
  halfAMinute: 's',
  lessThanXMinutes: 'm',
  xMinutes: 'm',
  aboutXHours: 'h',
  xHours: 'h',
  xDays: 'd',
  aboutXWeeks: 'w',
  xWeeks: 'w',
  aboutXMonths: 'mo',
  xMonths: 'mo',
  aboutXYears: 'y',
  xYears: 'y',
  overXYears: 'y',
  almostXYears: 'y',
}

export const formatRelativeDate = (date: Date | number | string): string => {
  return formatDistanceToNowStrict(date, {
    locale: {
      ...locale,
      formatDistance: (token, count) =>
        (token === 'halfAMinute' ? 30 : count) + formatDistanceLocale[token],
    },
  })
}
