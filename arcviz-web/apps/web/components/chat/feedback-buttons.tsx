'use client';

interface FeedbackButtonsProps {
  messageId: string;
  currentRating?: number;
  onFeedback: (messageId: string, rating: number) => void;
  onRequestCorrection: () => void;
}

export function FeedbackButtons({
  messageId,
  currentRating,
  onFeedback,
  onRequestCorrection,
}: FeedbackButtonsProps) {
  return (
    <div className="flex items-center space-x-2">
      <button
        onClick={() => onFeedback(messageId, 5)}
        className={`p-2 rounded-lg transition-colors ${
          currentRating === 5
            ? 'bg-green-100 text-green-600'
            : 'bg-gray-100 text-gray-600 hover:bg-green-50 hover:text-green-600'
        }`}
        title="Helpful"
      >
        <svg
          className="w-4 h-4"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path d="M2 10.5a1.5 1.5 0 113 0v6a1.5 1.5 0 01-3 0v-6zM6 10.333v5.43a2 2 0 001.106 1.79l.05.025A4 4 0 008.943 18h5.416a2 2 0 001.962-1.608l1.2-6A2 2 0 0015.56 8H12V4a2 2 0 00-2-2 1 1 0 00-1 1v.667a4 4 0 01-.8 2.4L6.8 7.933a4 4 0 00-.8 2.4z" />
        </svg>
      </button>

      <button
        onClick={() => onFeedback(messageId, 1)}
        className={`p-2 rounded-lg transition-colors ${
          currentRating === 1
            ? 'bg-red-100 text-red-600'
            : 'bg-gray-100 text-gray-600 hover:bg-red-50 hover:text-red-600'
        }`}
        title="Not helpful"
      >
        <svg
          className="w-4 h-4"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path d="M18 9.5a1.5 1.5 0 11-3 0v-6a1.5 1.5 0 013 0v6zM14 9.667v-5.43a2 2 0 00-1.105-1.79l-.05-.025A4 4 0 0011.055 2H5.64a2 2 0 00-1.962 1.608l-1.2 6A2 2 0 004.44 12H8v4a2 2 0 002 2 1 1 0 001-1v-.667a4 4 0 01.8-2.4l1.4-1.866a4 4 0 00.8-2.4z" />
        </svg>
      </button>

      <div className="w-px h-6 bg-gray-300" />

      <button
        onClick={onRequestCorrection}
        className="px-3 py-2 text-xs font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
      >
        <span className="flex items-center space-x-1">
          <svg
            className="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
            />
          </svg>
          <span>Correct</span>
        </span>
      </button>
    </div>
  );
}
