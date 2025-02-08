import { useEffect, useRef } from 'react'

type Props = {
  options: IntersectionObserverInit
  onChange: (isIntersecting: boolean) => void
}

export const useIntersectionObserver = (props: Props) => {
  const ref = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const element = ref.current
    if (!element) return

    const observer = new IntersectionObserver(async (entries) => {
      const entry = entries.at(0)
      if (entry) {
        props.onChange(entry.isIntersecting)
      }
    }, props.options)

    observer.observe(element)

    return () => observer.unobserve(element)
  }, [ref, props])

  return ref
}
