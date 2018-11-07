using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace YouthCenterSignIn.Logic.Data
{
    public class Log
    {
        #region Logs

        static Dictionary<string, IEnumerable<Log>> logsCache = new Dictionary<string, IEnumerable<Log>>();

        public static async Task<IEnumerable<Log>> GetLogs(DateTime date)
        {
            string file = GetLogsFileNameForDate(date);
            if (!logsCache.TryGetValue(file, out var datesLogsCache))
            {
                var logs = await DataProvider.Current.GetSetting<List<Log>>(file, StorageType.LocalFile) ?? new List<Log>();
                logsCache.Add(file, logs.OrderBy(l => l.SignedIn).ThenBy(l => l.PersonName));
            }

            return logsCache[file];
        }

        static async Task SaveLogs(IEnumerable<Log> logs)
        {
            var currentDate = logs.FirstOrDefault().SignInTime.Date;

            if (logs.Any(l => l.SignInTime.Date != currentDate))
                throw new InvalidOperationException("The list of logs to save contains logs that have different dates specified.  Please make sure the logs being saved are all the same date.");

            string file = GetLogsFileNameForDate(currentDate);
            await DataProvider.Current.SetSetting(file, logs, StorageType.LocalFile);

            if (logsCache.ContainsKey(file))
                logsCache[file] = logs;
            else
                logsCache.Add(file, logs);
        }

        private static string GetLogsFileNameForDate(DateTime date)
        {
            return $"{date.ToString("yyyy-MM-dd")}.json";
        }

        #endregion

        private Log()
        { } //Do nothing, private so the New method is what has to be used

        public static async Task<Log> New(Person person)
        {
            var log = new Log
            {
                PersonId = person.Id,
                PersonName = person.FullName,
                SignInTime = DateTime.Now,
                Guid = Guid.NewGuid()
            };

            var logs = (await GetLogs(log.SignInTime)).ToList();
            logs.Add(log);
            await SaveLogs(logs);

            return log;
        }

        public async Task SignOut()
        {
            SignOutTime = DateTime.Now;
            await Save();
        }

        async Task Save()
        {
            var logs = (await GetLogs(SignInTime)).ToList();

            var oldLog = logs.SingleOrDefault(p => p.Guid == Guid);
            if (oldLog != null)
                logs.Remove(oldLog);

            logs.Add(this);

            await SaveLogs(logs);
        }

        public Guid Guid { get; set; }

        public string PersonId { get; set; }

        public string PersonName { get; set; }

        public DateTime SignInTime { get; set; }

        public DateTime? SignOutTime { get; set; }

        public bool SignedIn => SignOutTime == null;
    }
}
