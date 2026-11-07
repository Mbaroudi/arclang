# ArcLang: the samples of a Monte Carlo case, analyzed with R.
#
# Usage: Rscript analysis.R samples.csv statistics.json
#
# samples.csv is written by `arclang verify`: one row per sample, the
# uncertain parameters in SI units, then one "margin: <expectation>" column
# per expectation (positive on the side that holds), then holds (1 or 0).
#
# For each margin: the share of samples that fail, a normal fit of the margin
# and the failure probability it extrapolates, and the partial rank
# correlation (PRCC) and standardized rank regression coefficient (SRRC) of
# each parameter, with 95 % bootstrap intervals. Base R only: no package to
# install. The verdict of the case is not computed here: it is the compiler's.

arguments <- commandArgs(trailingOnly = TRUE)
samples <- read.csv(arguments[1], check.names = FALSE, stringsAsFactors = FALSE)
margin_columns <- grep("^margin: ", names(samples))
inputs <- names(samples)[2:(margin_columns[1] - 1)]

json_string <- function(text) {
  text <- gsub("\\\\", "\\\\\\\\", text)
  text <- gsub("\"", "\\\\\"", text)
  paste0("\"", text, "\"")
}
json_number <- function(x) if (length(x) == 0 || is.na(x) || !is.finite(x)) "null" else format(signif(x, 6), scientific = FALSE, trim = TRUE)

# PRCC: the correlation of the residuals of the ranks of x_j and of y, each
# regressed on the ranks of the other parameters.
prcc <- function(x, y) {
  rx <- apply(x, 2, rank); ry <- rank(y)
  sapply(seq_len(ncol(rx)), function(j) {
    others <- cbind(1, rx[, -j, drop = FALSE])
    ex <- lm.fit(others, rx[, j])$residuals
    ey <- lm.fit(others, ry)$residuals
    # When the other parameters already order the margin (or x_j) entirely,
    # what is left is rounding: the correlation is undefined, not a number.
    if (sum(ex^2) < 1e-9 * sum((rx[, j] - mean(rx[, j]))^2) || sum(ey^2) < 1e-9 * sum((ry - mean(ry))^2)) NA else cor(ex, ey)
  })
}

# SRRC: the coefficients of the regression of the rank of y on the ranks of
# the parameters, each scaled by sd(x_j) / sd(y).
srrc <- function(x, y) {
  rx <- apply(x, 2, rank); ry <- rank(y)
  coefficients <- lm.fit(cbind(1, rx), ry)$coefficients[-1]
  coefficients * apply(rx, 2, sd) / sd(ry)
}

influence <- function(parameter, method, value, low, high) {
  sprintf("{\"parameter\": %s, \"method\": %s, \"value\": %s, \"low\": %s, \"high\": %s}",
          json_string(parameter), json_string(method), json_number(value), json_number(low), json_number(high))
}

set.seed(1)
margins <- character(0)
for (column in margin_columns) {
  kept <- !is.na(samples[[column]])
  y <- samples[[column]][kept]
  x <- as.matrix(samples[kept, inputs, drop = FALSE])
  m <- mean(y); s <- sd(y)
  tail <- if (s == 0) as.numeric(m < 0) else pnorm(0, mean = m, sd = s)
  # A margin that does not vary (a run ending on its bound) has no influence to explain.
  constant <- diff(range(y)) <= 1e-6 * max(abs(y), 1)
  influences <- character(0)
  if (!constant && length(y) >= 10) {
    p <- prcc(x, y); r <- srrc(x, y)
    boot <- replicate(200, { pick <- sample.int(length(y), replace = TRUE); c(prcc(x[pick, , drop = FALSE], y[pick]), srrc(x[pick, , drop = FALSE], y[pick])) })
    k <- length(inputs)
    for (j in seq_len(k)) {
      bp <- quantile(boot[j, ], c(0.025, 0.975), na.rm = TRUE)
      br <- quantile(boot[k + j, ], c(0.025, 0.975), na.rm = TRUE)
      influences <- c(influences, influence(inputs[j], "PRCC", p[j], bp[1], bp[2]), influence(inputs[j], "SRRC", r[j], br[1], br[2]))
    }
  }
  margins <- c(margins, sprintf(
    "{\"expect\": %s, \"samples\": %d, \"failure_fraction\": %s, \"margin_mean\": %s, \"margin_sd\": %s, \"fitted_failure_probability\": %s, \"constant\": %s, \"influences\": [%s]}",
    json_string(sub("^margin: ", "", names(samples)[column])), length(y), json_number(mean(y < 0)),
    json_number(m), json_number(s), json_number(tail), if (constant) "true" else "false", paste(influences, collapse = ", ")))
}
writeLines(sprintf("{\"engine\": %s, \"margins\": [%s]}", json_string(R.version.string), paste(margins, collapse = ", ")), arguments[2])
