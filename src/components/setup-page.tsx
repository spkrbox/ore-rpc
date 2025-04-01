interface SetupPageProps {
    userId: string
    onUserIdChange: (userId: string) => void
    onSave: () => void
}

export const SetupPage = ({ userId, onUserIdChange, onSave }: SetupPageProps) => (
    <div className='setup-page'>
        <div className='header'>
            <h1>Ore Presence</h1>
        </div>
        <p>Enter your User ID:</p>
        <div className='row'>
            <input
                id='userIdInput'
                onChange={(e) => onUserIdChange(e.currentTarget.value)}
                value={userId}
                placeholder='User ID'
            />
            <button onClick={onSave} type='button'>
                Save
            </button>
        </div>
    </div>
)
