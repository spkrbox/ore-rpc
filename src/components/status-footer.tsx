interface StatusFooterProps {
    status: string
}

export const StatusFooter = ({ status }: StatusFooterProps) => (
    <div className='status-footer'>
        <p className='status'>{status}</p>
    </div>
)
