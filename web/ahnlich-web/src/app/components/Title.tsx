import React, { ReactNode } from 'react'

export const Title = ({ children }: { children: ReactNode }) => {
  return (
    <h3 className="text-primary text-center font-medium text-3xl font-[family-name:var(--font-black-ops-one)] my-3">
      {children}
    </h3>
  )
}
