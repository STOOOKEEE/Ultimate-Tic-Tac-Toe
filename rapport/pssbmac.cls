\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{pssbmac}
\LoadClass[10pt, a4paper, twoside]{article}
\setlength{\textwidth}{15cm}
\setlength{\textheight}{21.0cm}
\setlength{\topmargin}{0cm}
\setlength{\oddsidemargin}{0.46cm}
\setlength{\evensidemargin}{0.46cm}
\renewenvironment{abstract}
{\begin{list}{}{%
  \setlength{\rightmargin}{0.5cm}
  \setlength{\leftmargin}{0.5cm}
  \small}
  \item[] }
{\end{list}}
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
\newcommand{\ps@headreport}
{ \renewcommand{\@oddhead}
  {\begin{minipage}{\textwidth}
     {\normalsize {\bf ESILV -- Groupe FISEA}\hfill Mai 2026}\\
     \rule{\textwidth}{0.6pt}
   \end{minipage}
  }
  \renewcommand{\@oddfoot}{}
  \renewcommand{\@evenhead}{\@oddhead}
  \renewcommand{\@evenfoot}{\@oddfoot}
}
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
\newcommand\criartitulo{\par
  \begingroup
    \renewcommand\thefootnote{\@arabic\c@footnote}%
    \def\@makefnmark{\rlap{\@textsuperscript{\normalfont\@thefnmark}}}%
    \long\def\@makefntext##1{\parindent 1em\noindent
            \hb@xt@1.8em{%
                \hss\@textsuperscript{\normalfont\@thefnmark}}##1}%
      \newpage
      \global\@topnum\z@
      \@criartitulo
    \thispagestyle{headreport}\@thanks
     \endgroup
  \global\let\thanks\relax
  \global\let\criartitulo\relax
  \global\let\@criartitulo\relax
  \global\let\@thanks\@empty
  \global\let\@author\@empty
  \global\let\@date\@empty
  \global\let\@title\@empty
  \global\let\title\relax
  \global\let\author\relax
  \global\let\date\relax
  \global\let\and\relax
}
\def\@criartitulo{%
  \newpage
  \null
  \vspace{-1em}%
  \begin{center}%
  \let \footnote \thanks
    {\LARGE \@title \par}%
    \vskip 1.5em%
    {\normalsize
      \lineskip .5em%
      \@author
    \par}%
    \vskip 1em%
  \end{center}%
  \par
  \vskip 1em}

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
\newcommand{\runningheads}{\markboth}
\pagestyle{headreport}
\normalsize
\setcounter{page}{1}
\endinput
