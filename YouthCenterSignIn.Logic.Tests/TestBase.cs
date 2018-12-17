using System.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class TestBase
    {
        protected TestDataProvider DataProvider => (TestDataProvider)Data.DataProvider.Current;

        [TestInitialize]
        public void TestInit()
        {
            Data.DataProvider.Current = new TestDataProvider();
        }

        protected Person GetTestPerson()
        {
            var search = new PersonSearch
            {
                SearchText = "James"
            };

            search.SearchTask.Wait();
            return search.Single();
        }
    }
}
