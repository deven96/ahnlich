import React, { ReactNode } from 'react'

const Card = ({ title, children, style }: { title: string | ReactNode; children: ReactNode; style?: string}) => {
  const isTitleString = typeof(title) === 'string';

  return (
    <div className={`rounded-lg h-full shadow-xl max-w-screen ${style}`}>
      {isTitleString ? <h3 className='p-5 text-3xl font-bold text-secondary'>{title}</h3> : title}
      <div className="flex flex-col gap-10 cursor-pointer px-5 py-8 whitespace-pre-wrap">
        {children}
      </div>
    </div>
  )
}

export default Card