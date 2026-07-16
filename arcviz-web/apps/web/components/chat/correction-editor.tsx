'use client';

import { useState } from 'react';

interface CorrectionEditorProps {
  messageId: string;
  originalCode: string;
  onSubmit: (correctedCode: string, feedback: string, issueType: string) => void;
  onCancel: () => void;
}

export function CorrectionEditor({
  messageId,
  originalCode,
  onSubmit,
  onCancel,
}: CorrectionEditorProps) {
  const [correctedCode, setCorrectedCode] = useState(originalCode);
  const [feedback, setFeedback] = useState('');
  const [issueType, setIssueType] = useState('semantic');

  const handleSubmit = () => {
    if (feedback.trim()) {
      onSubmit(correctedCode, feedback, issueType);
    }
  };

  return (
    <div className="border rounded-lg p-4 bg-yellow-50 border-yellow-200">
      <h4 className="text-sm font-medium text-gray-900 mb-3">
        Submit Correction
      </h4>

      <div className="space-y-3">
        <div>
          <label className="block text-xs font-medium text-gray-700 mb-1">
            Issue Type
          </label>
          <select
            value={issueType}
            onChange={(e) => setIssueType(e.target.value)}
            className="w-full px-3 py-2 text-sm border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="syntax">Syntax Error</option>
            <option value="semantic">Semantic Error</option>
            <option value="missing">Missing Information</option>
            <option value="wrong">Wrong Approach</option>
            <option value="other">Other</option>
          </select>
        </div>

        <div>
          <label className="block text-xs font-medium text-gray-700 mb-1">
            What&apos;s wrong? (Explain the issue)
          </label>
          <textarea
            value={feedback}
            onChange={(e) => setFeedback(e.target.value)}
            placeholder="Describe the issue and how it should be fixed..."
            rows={3}
            className="w-full px-3 py-2 text-sm border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {originalCode && (
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Corrected Code (optional)
            </label>
            <textarea
              value={correctedCode}
              onChange={(e) => setCorrectedCode(e.target.value)}
              rows={6}
              className="w-full px-3 py-2 text-sm font-mono border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
            />
          </div>
        )}

        <div className="flex items-center justify-end space-x-2 pt-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border rounded-lg hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!feedback.trim()}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
          >
            Submit Correction
          </button>
        </div>
      </div>
    </div>
  );
}
